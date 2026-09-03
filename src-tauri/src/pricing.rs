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

#[cfg(test)]
mod tests {
    use super::{effective_price, PriceSnapshot, PriceSource};
    use crate::realm::Realm;
    use crate::storage::SqliteRepository;

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
}
