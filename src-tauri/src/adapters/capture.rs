use crate::{domain::CurrencyAmount, realm::Realm};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CaptureCandidate {
    pub realm_hint: Option<Realm>,
    pub entries: Vec<CurrencyAmount>,
    pub confidence: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StoredCaptureCandidate {
    pub id: i64,
    pub candidate: CaptureCandidate,
}

impl CaptureCandidate {
    pub fn new(realm_hint: Option<Realm>, entries: Vec<CurrencyAmount>, confidence: u8) -> Self {
        Self {
            realm_hint,
            entries,
            confidence,
        }
    }
}
