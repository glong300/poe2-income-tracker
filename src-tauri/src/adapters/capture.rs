use crate::{domain::CurrencyAmount, realm::Realm};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureCandidate {
    pub realm_hint: Option<Realm>,
    pub entries: Vec<CurrencyAmount>,
    pub confidence: u8,
}

impl CaptureCandidate {
    pub fn new(realm_hint: Option<Realm>, entries: Vec<CurrencyAmount>, confidence: u8) -> Self {
        Self { realm_hint, entries, confidence }
    }
}
