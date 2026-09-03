use std::path::Path;

use rusqlite::{params, Connection, Result};

use crate::domain::{AdjustmentKind, CurrencyAmount, Direction, LedgerAdjustment, Snapshot, SnapshotStatus};
use crate::adapters::capture::CaptureCandidate;
use crate::pricing::{effective_price, PriceSnapshot, PriceSource};
use crate::realm::Realm;

pub struct SqliteRepository {
    connection: Connection,
}

impl SqliteRepository {
    pub fn open(path: &Path) -> Result<Self> {
        let connection = Connection::open(path)?;
        connection.execute_batch(
            "
            PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS snapshots (
                id INTEGER PRIMARY KEY,
                realm TEXT NOT NULL DEFAULT 'international' CHECK (realm IN ('international', 'china')),
                captured_at TEXT NOT NULL,
                status TEXT NOT NULL CHECK (status IN ('valid', 'invalid'))
            );
            CREATE TABLE IF NOT EXISTS snapshot_entries (
                snapshot_id INTEGER NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,
                currency_id TEXT NOT NULL,
                quantity INTEGER NOT NULL CHECK (quantity >= 0),
                PRIMARY KEY (snapshot_id, currency_id)
            );
            CREATE TABLE IF NOT EXISTS profile (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                realm TEXT NOT NULL CHECK (realm IN ('international', 'china'))
            );
            INSERT OR IGNORE INTO profile (id, realm) VALUES (1, 'china');
            CREATE TABLE IF NOT EXISTS price_snapshots (
                id INTEGER PRIMARY KEY,
                realm TEXT NOT NULL CHECK (realm IN ('international', 'china')),
                currency_id TEXT NOT NULL,
                value INTEGER NOT NULL CHECK (value > 0),
                quoted_in TEXT NOT NULL,
                source TEXT NOT NULL CHECK (source IN ('automatic', 'manual')),
                captured_at TEXT NOT NULL,
                confirmed INTEGER NOT NULL CHECK (confirmed IN (0, 1))
            );
            CREATE TABLE IF NOT EXISTS ledger_adjustments (
                id INTEGER PRIMARY KEY,
                realm TEXT NOT NULL CHECK (realm IN ('international', 'china')),
                currency_id TEXT NOT NULL,
                quantity INTEGER NOT NULL CHECK (quantity > 0),
                direction TEXT NOT NULL CHECK (direction IN ('inflow', 'outflow')),
                kind TEXT NOT NULL CHECK (kind IN ('trade', 'exchange', 'crafting', 'other')),
                occurred_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS capture_candidates (
                id INTEGER PRIMARY KEY,
                realm_hint TEXT,
                entries_json TEXT NOT NULL,
                confidence INTEGER NOT NULL CHECK (confidence >= 0 AND confidence <= 100)
            );
            ",
        )?;
        migrate_snapshots_realm(&connection)?;

        Ok(Self { connection })
    }

    pub fn realm(&self) -> Result<Realm> {
        let value =
            self.connection
                .query_row("SELECT realm FROM profile WHERE id = 1", [], |row| {
                    row.get::<_, String>(0)
                })?;

        Ok(Realm::parse(&value).expect("profile constraint prevents an unknown realm"))
    }

    pub fn set_realm(&self, realm: Realm) -> Result<()> {
        self.connection.execute(
            "UPDATE profile SET realm = ?1 WHERE id = 1",
            params![realm.as_str()],
        )?;
        Ok(())
    }

    pub fn save_snapshot(&self, snapshot: &Snapshot) -> Result<()> {
        self.save_snapshot_in_realm(self.realm()?, snapshot)
    }

    pub fn save_snapshot_in_realm(&self, realm: Realm, snapshot: &Snapshot) -> Result<()> {
        self.connection.execute(
            "INSERT INTO snapshots (realm, captured_at, status) VALUES (?1, ?2, ?3)",
            params![
                realm.as_str(),
                snapshot.captured_at,
                status_name(snapshot.status)
            ],
        )?;
        let snapshot_id = self.connection.last_insert_rowid();

        for entry in &snapshot.entries {
            self.connection.execute(
                "INSERT INTO snapshot_entries (snapshot_id, currency_id, quantity) VALUES (?1, ?2, ?3)",
                params![snapshot_id, entry.currency_id, entry.quantity],
            )?;
        }

        Ok(())
    }

    pub fn list_snapshots(&self) -> Result<Vec<Snapshot>> {
        self.list_snapshots_in_realm(self.realm()?)
    }

    pub fn list_snapshots_in_realm(&self, realm: Realm) -> Result<Vec<Snapshot>> {
        let mut snapshot_statement = self.connection.prepare(
            "SELECT id, captured_at, status FROM snapshots WHERE realm = ?1 ORDER BY captured_at ASC, id ASC",
        )?;
        let rows = snapshot_statement.query_map(params![realm.as_str()], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;

        let mut snapshots = Vec::new();
        for row in rows {
            let (snapshot_id, captured_at, status) = row?;
            let mut entry_statement = self.connection.prepare(
                "SELECT currency_id, quantity FROM snapshot_entries WHERE snapshot_id = ?1 ORDER BY currency_id ASC",
            )?;
            let entries = entry_statement
                .query_map(params![snapshot_id], |entry_row| {
                    Ok(CurrencyAmount::new(
                        entry_row.get::<_, String>(0)?,
                        entry_row.get::<_, u64>(1)?,
                    ))
                })?
                .collect::<Result<Vec<_>>>()?;

            snapshots.push(match status.as_str() {
                "valid" => Snapshot::valid(captured_at, entries),
                "invalid" => Snapshot::invalid(captured_at, entries),
                _ => unreachable!("database status constraint prevents unknown values"),
            });
        }

        Ok(snapshots)
    }

    pub fn save_price_snapshot(&self, price: &PriceSnapshot) -> Result<()> {
        self.connection.execute(
            "INSERT INTO price_snapshots (realm, currency_id, value, quoted_in, source, captured_at, confirmed) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                price.realm.as_str(),
                price.currency_id,
                price.value as i64,
                price.quoted_in,
                price_source_name(price.source),
                price.captured_at,
                price.confirmed,
            ],
        )?;
        Ok(())
    }

    pub fn effective_price(
        &self,
        realm: Realm,
        currency_id: &str,
        captured_at: &str,
    ) -> Result<Option<PriceSnapshot>> {
        let mut statement = self.connection.prepare(
            "SELECT realm, currency_id, value, quoted_in, source, captured_at, confirmed FROM price_snapshots WHERE realm = ?1 AND currency_id = ?2 AND captured_at = ?3",
        )?;
        let prices = statement
            .query_map(params![realm.as_str(), currency_id, captured_at], |row| {
                let source = match row.get::<_, String>(4)?.as_str() {
                    "automatic" => PriceSource::Automatic,
                    "manual" => PriceSource::Manual,
                    _ => unreachable!("database source constraint prevents unknown sources"),
                };
                Ok(PriceSnapshot::new(
                    Realm::parse(&row.get::<_, String>(0)?).expect("database realm constraint prevents unknown realms"),
                    row.get::<_, String>(1)?,
                    row.get::<_, u64>(2)?,
                    row.get::<_, String>(3)?,
                    source,
                    row.get::<_, String>(5)?,
                    row.get::<_, bool>(6)?,
                ))
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(effective_price(&prices, realm, currency_id, captured_at).cloned())
    }

    pub fn save_adjustment_in_realm(&self, realm: Realm, adjustment: &LedgerAdjustment) -> Result<()> {
        self.connection.execute(
            "INSERT INTO ledger_adjustments (realm, currency_id, quantity, direction, kind, occurred_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![realm.as_str(), adjustment.currency_id, adjustment.quantity as i64, direction_name(adjustment.direction), adjustment_kind_name(adjustment.kind), adjustment.occurred_at],
        )?;
        Ok(())
    }

    pub fn save_capture_candidate(&self, candidate: &CaptureCandidate) -> Result<()> {
        self.connection.execute(
            "INSERT INTO capture_candidates (realm_hint, entries_json, confidence) VALUES (?1, ?2, ?3)",
            params![candidate.realm_hint.map(Realm::as_str), serde_json::to_string(&candidate.entries).unwrap(), candidate.confidence],
        )?;
        Ok(())
    }

    pub fn list_adjustments_in_realm(&self, realm: Realm) -> Result<Vec<LedgerAdjustment>> {
        let mut statement = self.connection.prepare(
            "SELECT occurred_at, currency_id, quantity, direction, kind FROM ledger_adjustments WHERE realm = ?1 ORDER BY occurred_at ASC, id ASC",
        )?;
        let adjustments = statement.query_map(params![realm.as_str()], |row| {
            let direction = match row.get::<_, String>(3)?.as_str() { "inflow" => Direction::Inflow, "outflow" => Direction::Outflow, _ => unreachable!() };
            let kind = match row.get::<_, String>(4)?.as_str() { "trade" => AdjustmentKind::Trade, "exchange" => AdjustmentKind::Exchange, "crafting" => AdjustmentKind::Crafting, "other" => AdjustmentKind::Other, _ => unreachable!() };
            Ok(LedgerAdjustment::new(row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, u64>(2)?, direction, kind))
        })?.collect::<Result<Vec<_>>>()?;
        Ok(adjustments)
    }
}

fn migrate_snapshots_realm(connection: &Connection) -> Result<()> {
    let mut statement = connection.prepare("PRAGMA table_info(snapshots)")?;
    let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
    let mut has_realm = false;
    for column in columns {
        if column? == "realm" {
            has_realm = true;
            break;
        }
    }

    if !has_realm {
        connection.execute(
            "ALTER TABLE snapshots ADD COLUMN realm TEXT NOT NULL DEFAULT 'international' CHECK (realm IN ('international', 'china'))",
            [],
        )?;
    }

    Ok(())
}

fn status_name(status: SnapshotStatus) -> &'static str {
    match status {
        SnapshotStatus::Valid => "valid",
        SnapshotStatus::Invalid => "invalid",
    }
}

fn price_source_name(source: PriceSource) -> &'static str {
    match source {
        PriceSource::Automatic => "automatic",
        PriceSource::Manual => "manual",
    }
}

fn direction_name(direction: Direction) -> &'static str { match direction { Direction::Inflow => "inflow", Direction::Outflow => "outflow" } }
fn adjustment_kind_name(kind: AdjustmentKind) -> &'static str { match kind { AdjustmentKind::Trade => "trade", AdjustmentKind::Exchange => "exchange", AdjustmentKind::Crafting => "crafting", AdjustmentKind::Other => "other" } }
