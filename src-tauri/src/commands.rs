use serde::{Deserialize, Serialize};

use crate::domain::{AdjustmentKind, CurrencyAmount, Direction, LedgerAdjustment, Snapshot};
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

#[derive(Debug, Deserialize)]
pub struct AdjustmentInput {
    pub occurred_at: String,
    pub currency_id: String,
    pub quantity: i64,
    pub direction: String,
    pub kind: String,
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

pub fn create_adjustment(repository: &SqliteRepository, input: AdjustmentInput) -> Result<(), CommandError> {
    let quantity = u64::try_from(input.quantity).ok().filter(|quantity| *quantity > 0).ok_or_else(|| CommandError { code: "invalid_quantity", message: "数量必须为正整数".into() })?;
    let direction = match input.direction.as_str() { "inflow" => Direction::Inflow, "outflow" => Direction::Outflow, _ => return Err(CommandError { code: "invalid_direction", message: "方向必须为 inflow 或 outflow".into() }) };
    let kind = match input.kind.as_str() { "trade" => AdjustmentKind::Trade, "exchange" => AdjustmentKind::Exchange, "crafting" => AdjustmentKind::Crafting, "other" => AdjustmentKind::Other, _ => return Err(CommandError { code: "invalid_kind", message: "收支类型无效".into() }) };
    let realm = repository.realm().map_err(|_| CommandError { code: "storage_error", message: "无法读取当前区服".into() })?;
    repository.save_adjustment_in_realm(realm, &LedgerAdjustment::new(input.occurred_at, input.currency_id, quantity, direction, kind)).map_err(|_| CommandError { code: "storage_error", message: "无法保存收支调整".into() })
}
