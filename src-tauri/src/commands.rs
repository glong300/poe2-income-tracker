use serde::{Deserialize, Serialize};

use crate::domain::{CurrencyAmount, Snapshot};
use crate::storage::SqliteRepository;

#[derive(Debug, Deserialize)]
pub struct CurrencyQuantityInput {
    pub currency_id: String,
    pub quantity: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateSnapshotInput {
    pub captured_at: String,
    pub entries: Vec<CurrencyQuantityInput>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct CommandError {
    pub code: &'static str,
    pub message: String,
}

pub fn validate_snapshot_input(input: CreateSnapshotInput) -> Result<Snapshot, CommandError> {
    let mut entries = Vec::with_capacity(input.entries.len());
    for entry in input.entries {
        let quantity = u64::try_from(entry.quantity).map_err(|_| CommandError {
            code: "invalid_quantity",
            message: format!("数量必须是非负整数：{}", entry.currency_id),
        })?;
        entries.push(CurrencyAmount::new(entry.currency_id, quantity));
    }

    Snapshot::validated(input.captured_at, entries).map_err(|message| CommandError {
        code: "invalid_snapshot",
        message,
    })
}

pub fn create_snapshot(
    repository: &SqliteRepository,
    input: CreateSnapshotInput,
) -> Result<(), CommandError> {
    let snapshot = validate_snapshot_input(input)?;
    repository.save_snapshot(&snapshot).map_err(|_| CommandError {
        code: "storage_error",
        message: "无法保存本地快照".into(),
    })
}
