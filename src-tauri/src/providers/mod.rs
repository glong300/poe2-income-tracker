use serde::Serialize;

use crate::realm::Realm;

pub mod cn_market;
pub mod poe_ninja;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ProviderId {
    PoeNinja,
    CNMarket,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ProviderAvailability {
    AwaitingSync,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProviderStatus {
    pub provider: ProviderId,
    pub availability: ProviderAvailability,
    pub message: &'static str,
}

pub fn provider_status(realm: Realm) -> ProviderStatus {
    match realm {
        Realm::International => ProviderStatus {
            provider: ProviderId::PoeNinja,
            availability: ProviderAvailability::AwaitingSync,
            message: "国际服行情等待同步",
        },
        Realm::China => ProviderStatus {
            provider: ProviderId::CNMarket,
            availability: ProviderAvailability::Unavailable,
            message: "国服行情数据源尚未配置",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        cn_market::CNMarketProvider, poe_ninja::PoeNinjaProvider, provider_status,
        ProviderAvailability, ProviderId,
    };
    use crate::realm::Realm;

    #[test]
    fn marks_the_china_market_provider_unavailable_until_a_verified_endpoint_exists() {
        let status = provider_status(Realm::China);

        assert_eq!(status.provider, ProviderId::CNMarket);
        assert_eq!(status.availability, ProviderAvailability::Unavailable);
        assert_eq!(status.message, "国服行情数据源尚未配置");
    }

    #[test]
    fn keeps_the_two_provider_boundaries_realm_specific() {
        assert_eq!(PoeNinjaProvider::realm(), Realm::International);
        assert_eq!(CNMarketProvider::realm(), Realm::China);
    }
}
