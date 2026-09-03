use crate::realm::Realm;

pub struct CNMarketProvider;

impl CNMarketProvider {
    pub fn realm() -> Realm {
        Realm::China
    }
}
