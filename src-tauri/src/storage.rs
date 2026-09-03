use std::path::Path;

use rusqlite::{params, Connection, Result};

use crate::domain::{CurrencyAmount, Snapshot, SnapshotStatus};
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
            INSERT OR IGNORE INTO profile (id, realm) VALUES (1, 'international');
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
