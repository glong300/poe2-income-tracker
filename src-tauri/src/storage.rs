use std::path::Path;

use rusqlite::{params, Connection, Result};

use crate::domain::{CurrencyAmount, Snapshot, SnapshotStatus};

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
                captured_at TEXT NOT NULL,
                status TEXT NOT NULL CHECK (status IN ('valid', 'invalid'))
            );
            CREATE TABLE IF NOT EXISTS snapshot_entries (
                snapshot_id INTEGER NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,
                currency_id TEXT NOT NULL,
                quantity INTEGER NOT NULL CHECK (quantity >= 0),
                PRIMARY KEY (snapshot_id, currency_id)
            );
            ",
        )?;

        Ok(Self { connection })
    }

    pub fn save_snapshot(&self, snapshot: &Snapshot) -> Result<()> {
        self.connection.execute(
            "INSERT INTO snapshots (captured_at, status) VALUES (?1, ?2)",
            params![snapshot.captured_at, status_name(snapshot.status)],
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
        let mut snapshot_statement = self.connection.prepare(
            "SELECT id, captured_at, status FROM snapshots ORDER BY captured_at ASC, id ASC",
        )?;
        let rows = snapshot_statement.query_map([], |row| {
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

fn status_name(status: SnapshotStatus) -> &'static str {
    match status {
        SnapshotStatus::Valid => "valid",
        SnapshotStatus::Invalid => "invalid",
    }
}
