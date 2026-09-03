use crate::realm::Realm;

pub struct PoeNinjaProvider;

impl PoeNinjaProvider {
    pub fn realm() -> Realm {
        Realm::International
    }
}
