use serde::{Deserialize, Serialize};

use crate::realm::Realm;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PriceSource {
    Automatic,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceSnapshot {
    pub realm: Realm,
    pub currency_id: String,
    pub value: u64,
    pub quoted_in: String,
    pub source: PriceSource,
    pub captured_at: String,
    pub confirmed: bool,
}

impl PriceSnapshot {
    pub fn new(
        realm: Realm,
        currency_id: impl Into<String>,
        value: u64,
        quoted_in: impl Into<String>,
        source: PriceSource,
        captured_at: impl Into<String>,
        confirmed: bool,
    ) -> Self {
        Self {
            realm,
            currency_id: currency_id.into(),
            value,
            quoted_in: quoted_in.into(),
            source,
            captured_at: captured_at.into(),
            confirmed,
        }
    }
}

pub fn effective_price<'a>(
    prices: &'a [PriceSnapshot],
    realm: Realm,
    currency_id: &str,
    captured_at: &str,
) -> Option<&'a PriceSnapshot> {
    prices
        .iter()
        .filter(|price| {
            price.realm == realm
                && price.currency_id == currency_id
                && price.captured_at == captured_at
        })
        .max_by_key(|price| matches!(price.source, PriceSource::Manual) && price.confirmed)
}

pub fn parse_manual_price_csv(realm: Realm, csv: &str) -> Result<Vec<PriceSnapshot>, String> {
    let mut lines = csv.lines();
    if lines.next() != Some("currency_id,value,quoted_in,captured_at") {
        return Err("CSV 表头必须为 currency_id,value,quoted_in,captured_at".into());
    }

    let mut prices = Vec::new();
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let columns = line.split(',').map(str::trim).collect::<Vec<_>>();
        let row = index + 2;
        if columns.len() != 4 || columns.iter().any(|column| column.is_empty()) {
            return Err(format!("第 {row} 行格式无效"));
        }
        let value = columns[1]
            .parse::<u64>()
            .map_err(|_| format!("第 {row} 行价格必须为正整数"))?;
        if value == 0 {
            return Err(format!("第 {row} 行价格必须为正整数"));
        }

        prices.push(PriceSnapshot::new(
            realm,
            columns[0],
            value,
            columns[2],
            PriceSource::Manual,
            columns[3],
            true,
        ));
    }

    Ok(prices)
}

#[cfg(test)]
mod tests {
    use super::{effective_price, parse_manual_price_csv, PriceSnapshot, PriceSource};
    use crate::realm::Realm;
    use crate::storage::SqliteRepository;
    use crate::AppState;

    #[test]
    fn prefers_a_confirmed_manual_price_over_an_automatic_price_at_the_same_time() {
        let prices = vec![
            PriceSnapshot::new(
                Realm::International,
                "exalted",
                10,
                "chaos",
                PriceSource::Automatic,
                "2026-09-03T12:00:00+08:00",
                false,
            ),
            PriceSnapshot::new(
                Realm::International,
                "exalted",
                12,
                "chaos",
                PriceSource::Manual,
                "2026-09-03T12:00:00+08:00",
                true,
            ),
        ];

        let price = effective_price(
            &prices,
            Realm::International,
            "exalted",
            "2026-09-03T12:00:00+08:00",
        );

        assert_eq!(price.unwrap().value, 12);
    }

    #[test]
    fn persists_and_returns_the_effective_price_for_a_realm() {
        let path = std::env::temp_dir().join(format!("poe2-pricing-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let repository = SqliteRepository::open(&path).unwrap();
        let captured_at = "2026-09-03T12:00:00+08:00";

        repository
            .save_price_snapshot(&PriceSnapshot::new(
                Realm::International,
                "exalted",
                10,
                "chaos",
                PriceSource::Automatic,
                captured_at,
                false,
            ))
            .unwrap();
        repository
            .save_price_snapshot(&PriceSnapshot::new(
                Realm::International,
                "exalted",
                12,
                "chaos",
                PriceSource::Manual,
                captured_at,
                true,
            ))
            .unwrap();

        assert_eq!(
            repository
                .effective_price(Realm::International, "exalted", captured_at)
                .unwrap()
                .unwrap()
                .value,
            12
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn imports_manual_prices_for_the_selected_realm_from_csv() {
        let prices = parse_manual_price_csv(
            Realm::China,
            "currency_id,value,quoted_in,captured_at\nexalted,12,chaos,2026-09-03T12:00:00+08:00\n",
        )
        .unwrap();

        assert_eq!(prices.len(), 1);
        assert_eq!(prices[0].realm, Realm::China);
        assert_eq!(prices[0].source, PriceSource::Manual);
        assert!(prices[0].confirmed);
    }

    #[test]
    fn rejects_non_positive_price_rows_in_csv() {
        let result = parse_manual_price_csv(
            Realm::International,
            "currency_id,value,quoted_in,captured_at\nexalted,0,chaos,2026-09-03T12:00:00+08:00\n",
        );

        assert_eq!(result.unwrap_err(), "第 2 行价格必须为正整数");
    }

    #[test]
    fn imports_csv_prices_into_the_current_realm_ledger() {
        let path =
            std::env::temp_dir().join(format!("poe2-price-import-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let state = AppState::open(&path).unwrap();
        let captured_at = "2026-09-03T12:00:00+08:00";

        state
            .import_manual_prices(&format!(
                "currency_id,value,quoted_in,captured_at\nexalted,12,chaos,{captured_at}\n"
            ))
            .unwrap();

        let repository = SqliteRepository::open(&path).unwrap();
        assert_eq!(
            repository
                .effective_price(Realm::China, "exalted", captured_at)
                .unwrap()
                .unwrap()
                .value,
            12
        );
        std::fs::remove_file(path).unwrap();
    }
}
