use std::collections::BTreeMap;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrencyAmount {
    pub currency_id: String,
    pub quantity: u64,
}

impl CurrencyAmount {
    pub fn new(currency_id: impl Into<String>, quantity: u64) -> Self {
        Self {
            currency_id: currency_id.into(),
            quantity,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotStatus {
    Valid,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub captured_at: String,
    pub entries: Vec<CurrencyAmount>,
    pub status: SnapshotStatus,
}

impl Snapshot {
    pub fn validated(
        captured_at: impl Into<String>,
        entries: Vec<CurrencyAmount>,
    ) -> Result<Self, String> {
        let mut seen = std::collections::BTreeSet::new();
        for entry in &entries {
            if !seen.insert(entry.currency_id.clone()) {
                return Err(format!("duplicate currency entry: {}", entry.currency_id));
            }
        }

        Ok(Self::new(captured_at, entries, SnapshotStatus::Valid))
    }

    pub fn valid(captured_at: impl Into<String>, entries: Vec<CurrencyAmount>) -> Self {
        Self::new(captured_at, entries, SnapshotStatus::Valid)
    }

    pub fn invalid(captured_at: impl Into<String>, entries: Vec<CurrencyAmount>) -> Self {
        Self::new(captured_at, entries, SnapshotStatus::Invalid)
    }

    fn new(
        captured_at: impl Into<String>,
        entries: Vec<CurrencyAmount>,
        status: SnapshotStatus,
    ) -> Self {
        Self {
            captured_at: captured_at.into(),
            entries,
            status,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Inflow,
    Outflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjustmentKind {
    Trade,
    Exchange,
    Crafting,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerAdjustment {
    pub occurred_at: String,
    pub currency_id: String,
    pub quantity: u64,
    pub direction: Direction,
    pub kind: AdjustmentKind,
}

impl LedgerAdjustment {
    pub fn new(
        occurred_at: impl Into<String>,
        currency_id: impl Into<String>,
        quantity: u64,
        direction: Direction,
        kind: AdjustmentKind,
    ) -> Self {
        Self {
            occurred_at: occurred_at.into(),
            currency_id: currency_id.into(),
            quantity,
            direction,
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CurrencyDayLedger {
    pub currency_id: String,
    pub net_change: i64,
    pub explained_change: i64,
    pub unattributed_change: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    InvalidDay,
}

pub fn calculate_day(
    snapshots: &[Snapshot],
    adjustments: &[LedgerAdjustment],
    day: &str,
) -> Result<Vec<CurrencyDayLedger>, DomainError> {
    if day.len() != 10 || !day.chars().enumerate().all(|(index, character)| {
        matches!(index, 4 | 7) && character == '-' || !matches!(index, 4 | 7) && character.is_ascii_digit()
    }) {
        return Err(DomainError::InvalidDay);
    }

    let mut quantities_by_currency: BTreeMap<String, Vec<(&str, u64)>> = BTreeMap::new();
    for snapshot in snapshots.iter().filter(|snapshot| {
        snapshot.status == SnapshotStatus::Valid && snapshot.captured_at.starts_with(day)
    }) {
        for entry in &snapshot.entries {
            quantities_by_currency
                .entry(entry.currency_id.clone())
                .or_default()
                .push((&snapshot.captured_at, entry.quantity));
        }
    }

    let mut adjustments_by_currency: BTreeMap<String, i64> = BTreeMap::new();
    for adjustment in adjustments
        .iter()
        .filter(|adjustment| adjustment.occurred_at.starts_with(day))
    {
        let signed_quantity = match adjustment.direction {
            Direction::Inflow => adjustment.quantity as i64,
            Direction::Outflow => -(adjustment.quantity as i64),
        };
        *adjustments_by_currency
            .entry(adjustment.currency_id.clone())
            .or_default() += signed_quantity;
    }

    let mut rows = Vec::new();
    for (currency_id, quantities) in &mut quantities_by_currency {
        quantities.sort_by_key(|(captured_at, _)| *captured_at);
        if let (Some((_, first)), Some((_, last))) = (quantities.first(), quantities.last()) {
            if quantities.len() < 2 {
                continue;
            }
            let net_change = *last as i64 - *first as i64;
            let explained_change = *adjustments_by_currency.get(currency_id).unwrap_or(&0);
            rows.push(CurrencyDayLedger {
                currency_id: currency_id.clone(),
                net_change,
                explained_change,
                unattributed_change: net_change - explained_change,
            });
        }
    }

    Ok(rows)
}

pub fn calculate_week(snapshots: &[Snapshot], adjustments: &[LedgerAdjustment], week_start: &str) -> Result<Vec<CurrencyDayLedger>, DomainError> {
    let start = date_key(week_start).ok_or(DomainError::InvalidDay)?;
    let in_week = |timestamp: &str| date_key(&timestamp[..timestamp.len().min(10)]).is_some_and(|date| date >= start && date < start + 7);
    let mut quantities_by_currency: BTreeMap<String, Vec<(&str, u64)>> = BTreeMap::new();
    for snapshot in snapshots.iter().filter(|snapshot| snapshot.status == SnapshotStatus::Valid && in_week(&snapshot.captured_at)) {
        for entry in &snapshot.entries { quantities_by_currency.entry(entry.currency_id.clone()).or_default().push((&snapshot.captured_at, entry.quantity)); }
    }
    let mut adjustments_by_currency: BTreeMap<String, i64> = BTreeMap::new();
    for adjustment in adjustments.iter().filter(|adjustment| in_week(&adjustment.occurred_at)) {
        let signed = match adjustment.direction { Direction::Inflow => adjustment.quantity as i64, Direction::Outflow => -(adjustment.quantity as i64) };
        *adjustments_by_currency.entry(adjustment.currency_id.clone()).or_default() += signed;
    }
    let mut rows = Vec::new();
    for (currency_id, quantities) in &mut quantities_by_currency {
        quantities.sort_by_key(|(captured_at, _)| *captured_at);
        if quantities.len() >= 2 { let net = quantities.last().unwrap().1 as i64 - quantities.first().unwrap().1 as i64; let explained = *adjustments_by_currency.get(currency_id).unwrap_or(&0); rows.push(CurrencyDayLedger { currency_id: currency_id.clone(), net_change: net, explained_change: explained, unattributed_change: net - explained }); }
    }
    Ok(rows)
}

fn date_key(value: &str) -> Option<i32> {
    if value.len() != 10 { return None; }
    let year = value[0..4].parse::<i32>().ok()?; let month = value[5..7].parse::<i32>().ok()?; let day = value[8..10].parse::<i32>().ok()?;
    if value.as_bytes()[4] != b'-' || value.as_bytes()[7] != b'-' || !(1..=12).contains(&month) || !(1..=31).contains(&day) { return None; }
    let year = year - i32::from(month <= 2); let era = if year >= 0 { year } else { year - 399 } / 400; let yoe = year - era * 400; let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1; Some(era * 146097 + yoe * 365 + yoe / 4 - yoe / 100 + doy)
}
