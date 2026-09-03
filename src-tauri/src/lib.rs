// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::Manager;

pub mod adapters;
pub mod commands;
pub mod domain;
pub mod pricing;
pub mod providers;
pub mod realm;
pub mod storage;

pub struct AppState {
    repository: std::sync::Mutex<storage::SqliteRepository>,
}

impl AppState {
    pub fn open(path: &std::path::Path) -> rusqlite::Result<Self> {
        Ok(Self {
            repository: std::sync::Mutex::new(storage::SqliteRepository::open(path)?),
        })
    }

    pub fn save_snapshot(
        &self,
        input: commands::CreateSnapshotInput,
    ) -> Result<(), commands::CommandError> {
        let repository = self.repository.lock().map_err(|_| commands::CommandError {
            code: "state_error",
            message: "本地账本不可用".into(),
        })?;
        commands::create_snapshot(&repository, input)
    }

    pub fn snapshot_count(&self) -> rusqlite::Result<usize> {
        Ok(self
            .repository
            .lock()
            .expect("repository lock is not poisoned")
            .list_snapshots()?
            .len())
    }

    pub fn realm(&self) -> rusqlite::Result<realm::Realm> {
        self.repository
            .lock()
            .expect("repository lock is not poisoned")
            .realm()
    }

    pub fn set_realm(&self, realm: realm::Realm) -> rusqlite::Result<()> {
        self.repository
            .lock()
            .expect("repository lock is not poisoned")
            .set_realm(realm)
    }

    pub fn price_provider_status(&self) -> rusqlite::Result<providers::ProviderStatus> {
        Ok(providers::provider_status(self.realm()?))
    }

    pub fn import_manual_prices(&self, csv: &str) -> Result<usize, commands::CommandError> {
        let repository = self.repository.lock().map_err(|_| commands::CommandError {
            code: "state_error",
            message: "本地账本不可用".into(),
        })?;
        let realm = repository.realm().map_err(|_| commands::CommandError {
            code: "storage_error",
            message: "无法读取当前区服".into(),
        })?;
        let prices = pricing::parse_manual_price_csv(realm, csv).map_err(|message| {
            commands::CommandError {
                code: "invalid_csv",
                message,
            }
        })?;
        for price in &prices {
            repository
                .save_price_snapshot(price)
                .map_err(|_| commands::CommandError {
                    code: "storage_error",
                    message: "无法保存手动价格".into(),
                })?;
        }
        Ok(prices.len())
    }

    pub fn save_adjustment(
        &self,
        adjustment: domain::LedgerAdjustment,
    ) -> Result<(), commands::CommandError> {
        let repository = self.repository.lock().map_err(|_| commands::CommandError {
            code: "state_error",
            message: "本地账本不可用".into(),
        })?;
        let realm = repository.realm().map_err(|_| commands::CommandError {
            code: "storage_error",
            message: "无法读取当前区服".into(),
        })?;
        repository
            .save_adjustment_in_realm(realm, &adjustment)
            .map_err(|_| commands::CommandError {
                code: "storage_error",
                message: "无法保存收支调整".into(),
            })
    }

    pub fn save_adjustment_input(
        &self,
        input: commands::AdjustmentInput,
    ) -> Result<(), commands::CommandError> {
        let repository = self.repository.lock().map_err(|_| commands::CommandError {
            code: "state_error",
            message: "本地账本不可用".into(),
        })?;
        commands::create_adjustment(&repository, input)
    }

    pub fn daily_ledger(
        &self,
        day: &str,
    ) -> Result<Vec<domain::CurrencyDayLedger>, commands::CommandError> {
        let repository = self.repository.lock().map_err(|_| commands::CommandError {
            code: "state_error",
            message: "本地账本不可用".into(),
        })?;
        let snapshots = repository
            .list_snapshots()
            .map_err(|_| commands::CommandError {
                code: "storage_error",
                message: "无法读取本地账本".into(),
            })?;
        let realm = repository.realm().map_err(|_| commands::CommandError {
            code: "storage_error",
            message: "无法读取当前区服".into(),
        })?;
        let adjustments =
            repository
                .list_adjustments_in_realm(realm)
                .map_err(|_| commands::CommandError {
                    code: "storage_error",
                    message: "无法读取收支调整".into(),
                })?;
        domain::calculate_day(&snapshots, &adjustments, day).map_err(|_| commands::CommandError {
            code: "invalid_day",
            message: "日期格式无效".into(),
        })
    }

    pub fn weekly_ledger(
        &self,
        week_start: &str,
    ) -> Result<Vec<domain::CurrencyDayLedger>, commands::CommandError> {
        let repository = self.repository.lock().map_err(|_| commands::CommandError {
            code: "state_error",
            message: "本地账本不可用".into(),
        })?;
        let snapshots = repository
            .list_snapshots()
            .map_err(|_| commands::CommandError {
                code: "storage_error",
                message: "无法读取本地账本".into(),
            })?;
        let realm = repository.realm().map_err(|_| commands::CommandError {
            code: "storage_error",
            message: "无法读取当前区服".into(),
        })?;
        let adjustments =
            repository
                .list_adjustments_in_realm(realm)
                .map_err(|_| commands::CommandError {
                    code: "storage_error",
                    message: "无法读取收支调整".into(),
                })?;
        domain::calculate_week(&snapshots, &adjustments, week_start).map_err(|_| {
            commands::CommandError {
                code: "invalid_day",
                message: "周起始日期无效".into(),
            }
        })
    }

    pub fn save_capture_candidate(
        &self,
        candidate: adapters::capture::CaptureCandidate,
    ) -> rusqlite::Result<()> {
        self.repository
            .lock()
            .expect("repository lock is not poisoned")
            .save_capture_candidate(&candidate)
    }

    pub fn capture_candidates(
        &self,
    ) -> rusqlite::Result<Vec<adapters::capture::StoredCaptureCandidate>> {
        self.repository
            .lock()
            .expect("repository lock is not poisoned")
            .list_capture_candidates()
    }

    pub fn confirm_capture_candidate(
        &self,
        candidate_id: i64,
        captured_at: &str,
    ) -> Result<(), commands::CommandError> {
        let repository = self.repository.lock().map_err(|_| commands::CommandError {
            code: "state_error",
            message: "本地账本不可用".into(),
        })?;
        let realm = repository.realm().map_err(|_| commands::CommandError {
            code: "storage_error",
            message: "无法读取当前区服".into(),
        })?;
        let confirmed = repository
            .confirm_capture_candidate(candidate_id, realm, captured_at)
            .map_err(|_| commands::CommandError {
                code: "storage_error",
                message: "无法确认本地候选".into(),
            })?;
        if confirmed {
            Ok(())
        } else {
            Err(commands::CommandError {
                code: "candidate_not_found",
                message: "候选记录不存在或已处理".into(),
            })
        }
    }

    pub fn confirm_capture_candidate_with_entries(
        &self,
        candidate_id: i64,
        captured_at: &str,
        entries: Vec<domain::CurrencyAmount>,
    ) -> Result<(), commands::CommandError> {
        let snapshot = domain::Snapshot::validated(captured_at, entries).map_err(|message| {
            commands::CommandError {
                code: "invalid_snapshot",
                message,
            }
        })?;
        let repository = self.repository.lock().map_err(|_| commands::CommandError {
            code: "state_error",
            message: "本地账本不可用".into(),
        })?;
        let realm = repository.realm().map_err(|_| commands::CommandError {
            code: "storage_error",
            message: "无法读取当前区服".into(),
        })?;
        let confirmed = repository
            .confirm_capture_candidate_with_entries(
                candidate_id,
                realm,
                &snapshot.captured_at,
                snapshot.entries,
            )
            .map_err(|_| commands::CommandError {
                code: "storage_error",
                message: "无法确认本地候选".into(),
            })?;
        if confirmed {
            Ok(())
        } else {
            Err(commands::CommandError {
                code: "candidate_not_found",
                message: "候选记录不存在或已处理".into(),
            })
        }
    }

    pub fn reject_capture_candidate(
        &self,
        candidate_id: i64,
    ) -> Result<(), commands::CommandError> {
        let repository = self.repository.lock().map_err(|_| commands::CommandError {
            code: "state_error",
            message: "本地账本不可用".into(),
        })?;
        if repository
            .delete_capture_candidate(candidate_id)
            .map_err(|_| commands::CommandError {
                code: "storage_error",
                message: "无法移除本地候选".into(),
            })?
        {
            Ok(())
        } else {
            Err(commands::CommandError {
                code: "candidate_not_found",
                message: "候选记录不存在或已处理".into(),
            })
        }
    }
}

#[tauri::command]
fn create_snapshot(
    state: tauri::State<'_, AppState>,
    input: commands::CreateSnapshotInput,
) -> Result<(), commands::CommandError> {
    state.save_snapshot(input)
}

#[tauri::command]
fn create_adjustment(
    state: tauri::State<'_, AppState>,
    input: commands::AdjustmentInput,
) -> Result<(), commands::CommandError> {
    state.save_adjustment_input(input)
}

#[tauri::command]
fn get_daily_ledger(
    state: tauri::State<'_, AppState>,
    day: String,
) -> Result<Vec<domain::CurrencyDayLedger>, commands::CommandError> {
    state.daily_ledger(&day)
}

#[tauri::command]
fn get_weekly_ledger(
    state: tauri::State<'_, AppState>,
    week_start: String,
) -> Result<Vec<domain::CurrencyDayLedger>, commands::CommandError> {
    state.weekly_ledger(&week_start)
}

fn current_realm(state: &AppState) -> Result<realm::Realm, commands::CommandError> {
    state.realm().map_err(|_| commands::CommandError {
        code: "storage_error",
        message: "无法读取当前区服".into(),
    })
}

fn change_realm(state: &AppState, value: &str) -> Result<(), commands::CommandError> {
    let realm = realm::Realm::parse(value).ok_or_else(|| commands::CommandError {
        code: "invalid_realm",
        message: "区服必须为 international 或 china".into(),
    })?;
    state.set_realm(realm).map_err(|_| commands::CommandError {
        code: "storage_error",
        message: "无法保存当前区服".into(),
    })
}

#[tauri::command]
fn get_realm(state: tauri::State<'_, AppState>) -> Result<realm::Realm, commands::CommandError> {
    current_realm(&state)
}

#[tauri::command]
fn set_realm(
    state: tauri::State<'_, AppState>,
    realm: String,
) -> Result<(), commands::CommandError> {
    change_realm(&state, &realm)
}

#[tauri::command]
fn get_price_provider_status(
    state: tauri::State<'_, AppState>,
) -> Result<providers::ProviderStatus, commands::CommandError> {
    state
        .price_provider_status()
        .map_err(|_| commands::CommandError {
            code: "storage_error",
            message: "无法读取行情数据源状态".into(),
        })
}

#[tauri::command]
fn import_manual_prices(
    state: tauri::State<'_, AppState>,
    csv: String,
) -> Result<usize, commands::CommandError> {
    state.import_manual_prices(&csv)
}

#[tauri::command]
fn get_capture_candidates(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<adapters::capture::StoredCaptureCandidate>, commands::CommandError> {
    state
        .capture_candidates()
        .map_err(|_| commands::CommandError {
            code: "storage_error",
            message: "无法读取本地候选".into(),
        })
}

#[tauri::command]
fn confirm_capture_candidate(
    state: tauri::State<'_, AppState>,
    candidate_id: i64,
    captured_at: String,
    entries: Vec<commands::CurrencyQuantityInput>,
) -> Result<(), commands::CommandError> {
    let snapshot = commands::validate_snapshot_input(commands::CreateSnapshotInput {
        captured_at,
        entries,
    })?;
    state.confirm_capture_candidate_with_entries(
        candidate_id,
        &snapshot.captured_at,
        snapshot.entries,
    )
}

#[tauri::command]
fn reject_capture_candidate(
    state: tauri::State<'_, AppState>,
    candidate_id: i64,
) -> Result<(), commands::CommandError> {
    state.reject_capture_candidate(candidate_id)
}

#[cfg(test)]
mod ledger_tests {
    use super::domain::{
        calculate_day, calculate_week, AdjustmentKind, CurrencyAmount, Direction, LedgerAdjustment,
        Snapshot,
    };

    #[test]
    fn calculates_net_explained_and_unattributed_change_for_one_currency() {
        let snapshots = vec![
            Snapshot::valid(
                "2026-09-03T09:00:00+08:00",
                vec![CurrencyAmount::new("exalted", 10)],
            ),
            Snapshot::valid(
                "2026-09-03T21:00:00+08:00",
                vec![CurrencyAmount::new("exalted", 17)],
            ),
        ];
        let adjustments = vec![LedgerAdjustment::new(
            "2026-09-03T12:00:00+08:00",
            "exalted",
            4,
            Direction::Inflow,
            AdjustmentKind::Trade,
        )];

        let rows = calculate_day(&snapshots, &adjustments, "2026-09-03").unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].currency_id, "exalted");
        assert_eq!(rows[0].net_change, 7);
        assert_eq!(rows[0].explained_change, 4);
        assert_eq!(rows[0].unattributed_change, 3);
    }

    #[test]
    fn excludes_invalid_snapshots_from_daily_calculation() {
        let snapshots = vec![
            Snapshot::valid(
                "2026-09-03T09:00:00+08:00",
                vec![CurrencyAmount::new("exalted", 10)],
            ),
            Snapshot::invalid(
                "2026-09-03T12:00:00+08:00",
                vec![CurrencyAmount::new("exalted", 999)],
            ),
            Snapshot::valid(
                "2026-09-03T21:00:00+08:00",
                vec![CurrencyAmount::new("exalted", 17)],
            ),
        ];

        let rows = calculate_day(&snapshots, &[], "2026-09-03").unwrap();

        assert_eq!(rows[0].net_change, 7);
    }

    #[test]
    fn rejects_duplicate_currency_entries_in_a_snapshot() {
        let result = Snapshot::validated(
            "2026-09-03T09:00:00+08:00",
            vec![
                CurrencyAmount::new("exalted", 1),
                CurrencyAmount::new("exalted", 2),
            ],
        );

        assert_eq!(result.unwrap_err(), "duplicate currency entry: exalted");
    }

    #[test]
    fn calculates_weekly_change_from_first_to_last_snapshot() {
        let snapshots = vec![
            Snapshot::valid(
                "2026-09-01T09:00:00+08:00",
                vec![CurrencyAmount::new("exalted", 10)],
            ),
            Snapshot::valid(
                "2026-09-07T21:00:00+08:00",
                vec![CurrencyAmount::new("exalted", 18)],
            ),
        ];
        let adjustments = vec![LedgerAdjustment::new(
            "2026-09-04T12:00:00+08:00",
            "exalted",
            3,
            Direction::Inflow,
            AdjustmentKind::Trade,
        )];

        let rows = calculate_week(&snapshots, &adjustments, "2026-09-01").unwrap();

        assert_eq!(rows[0].net_change, 8);
        assert_eq!(rows[0].explained_change, 3);
    }
}

#[cfg(test)]
mod sqlite_repository_tests {
    use super::domain::{calculate_day, CurrencyAmount, Snapshot};
    use super::storage::SqliteRepository;

    #[test]
    fn persists_snapshots_and_rebuilds_daily_ledger_after_reopening() {
        let database_path =
            std::env::temp_dir().join(format!("poe2-income-tracker-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&database_path);

        let repository = SqliteRepository::open(&database_path).unwrap();
        repository
            .save_snapshot(&Snapshot::valid(
                "2026-09-03T09:00:00+08:00",
                vec![CurrencyAmount::new("exalted", 10)],
            ))
            .unwrap();
        repository
            .save_snapshot(&Snapshot::valid(
                "2026-09-03T21:00:00+08:00",
                vec![CurrencyAmount::new("exalted", 17)],
            ))
            .unwrap();
        drop(repository);

        let reopened = SqliteRepository::open(&database_path).unwrap();
        let rows = calculate_day(&reopened.list_snapshots().unwrap(), &[], "2026-09-03").unwrap();

        assert_eq!(rows[0].net_change, 7);
        std::fs::remove_file(database_path).unwrap();
    }
}

#[cfg(test)]
mod command_tests {
    use super::commands::{
        create_adjustment, create_snapshot, validate_snapshot_input, AdjustmentInput,
        CreateSnapshotInput, CurrencyQuantityInput,
    };
    use super::domain::{AdjustmentKind, Direction};
    use super::storage::SqliteRepository;

    #[test]
    fn rejects_a_negative_quantity_at_the_command_boundary() {
        let result = validate_snapshot_input(CreateSnapshotInput {
            captured_at: "2026-09-03T09:00:00+08:00".into(),
            entries: vec![CurrencyQuantityInput {
                currency_id: "exalted".into(),
                quantity: -1,
            }],
        });

        assert_eq!(result.unwrap_err().code, "invalid_quantity");
    }

    #[test]
    fn saves_a_validated_snapshot_through_the_command_service() {
        let database_path =
            std::env::temp_dir().join(format!("poe2-income-command-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&database_path);
        let repository = SqliteRepository::open(&database_path).unwrap();

        create_snapshot(
            &repository,
            CreateSnapshotInput {
                captured_at: "2026-09-03T09:00:00+08:00".into(),
                entries: vec![CurrencyQuantityInput {
                    currency_id: "exalted".into(),
                    quantity: 10,
                }],
            },
        )
        .unwrap();

        assert_eq!(repository.list_snapshots().unwrap().len(), 1);
        std::fs::remove_file(database_path).unwrap();
    }

    #[test]
    fn validates_and_saves_a_crafting_outflow_adjustment() {
        let database_path = std::env::temp_dir().join(format!(
            "poe2-adjustment-command-{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&database_path);
        let repository = SqliteRepository::open(&database_path).unwrap();

        create_adjustment(
            &repository,
            AdjustmentInput {
                occurred_at: "2026-09-03T12:00:00+08:00".into(),
                currency_id: "exalted".into(),
                quantity: 2,
                direction: "outflow".into(),
                kind: "crafting".into(),
            },
        )
        .unwrap();

        let adjustment = repository
            .list_adjustments_in_realm(super::realm::Realm::China)
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(adjustment.direction, Direction::Outflow);
        assert_eq!(adjustment.kind, AdjustmentKind::Crafting);
        std::fs::remove_file(database_path).unwrap();
    }
}

#[cfg(test)]
mod adjustment_state_tests {
    use super::{commands::AdjustmentInput, AppState};

    #[test]
    fn app_state_saves_an_adjustment_from_command_input() {
        let path = std::env::temp_dir().join(format!(
            "poe2-adjustment-state-{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let state = AppState::open(&path).unwrap();

        state
            .save_adjustment_input(AdjustmentInput {
                occurred_at: "2026-09-03T12:00:00+08:00".into(),
                currency_id: "exalted".into(),
                quantity: 2,
                direction: "inflow".into(),
                kind: "trade".into(),
            })
            .unwrap();

        assert_eq!(state.daily_ledger("2026-09-03").unwrap(), Vec::new());
        std::fs::remove_file(path).unwrap();
    }
}

#[cfg(test)]
mod capture_candidate_tests {
    use super::{
        adapters::capture::CaptureCandidate, domain::CurrencyAmount, realm::Realm,
        storage::SqliteRepository, AppState,
    };

    #[test]
    fn candidate_is_not_a_snapshot_until_confirmed() {
        let candidate = CaptureCandidate::new(
            Some(Realm::China),
            vec![CurrencyAmount::new("exalted", 12)],
            88,
        );

        assert_eq!(candidate.entries[0].quantity, 12);
        assert_eq!(candidate.realm_hint, Some(Realm::China));
    }

    #[test]
    fn storing_a_candidate_does_not_create_a_snapshot() {
        let path =
            std::env::temp_dir().join(format!("poe2-candidate-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let repository = SqliteRepository::open(&path).unwrap();

        repository
            .save_capture_candidate(&CaptureCandidate::new(
                Some(Realm::China),
                vec![CurrencyAmount::new("exalted", 12)],
                88,
            ))
            .unwrap();

        assert_eq!(
            repository
                .list_snapshots_in_realm(Realm::China)
                .unwrap()
                .len(),
            0
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn confirming_a_candidate_creates_a_snapshot_and_removes_the_candidate() {
        let path = std::env::temp_dir().join(format!(
            "poe2-candidate-confirm-{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let state = AppState::open(&path).unwrap();
        state
            .save_capture_candidate(CaptureCandidate::new(
                Some(Realm::China),
                vec![CurrencyAmount::new("exalted", 12)],
                88,
            ))
            .unwrap();

        let candidate = state.capture_candidates().unwrap().pop().unwrap();
        state
            .confirm_capture_candidate(candidate.id, "2026-09-03T12:00:00+08:00")
            .unwrap();

        assert!(state.capture_candidates().unwrap().is_empty());
        assert_eq!(state.snapshot_count().unwrap(), 1);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn confirming_a_candidate_uses_the_reviewed_quantities() {
        let path = std::env::temp_dir().join(format!(
            "poe2-candidate-edited-{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let state = AppState::open(&path).unwrap();
        state
            .save_capture_candidate(CaptureCandidate::new(
                Some(Realm::China),
                vec![CurrencyAmount::new("exalted", 12)],
                88,
            ))
            .unwrap();

        let candidate = state.capture_candidates().unwrap().pop().unwrap();
        state
            .confirm_capture_candidate_with_entries(
                candidate.id,
                "2026-09-03T12:00:00+08:00",
                vec![CurrencyAmount::new("exalted", 14)],
            )
            .unwrap();

        let repository = SqliteRepository::open(&path).unwrap();
        assert_eq!(
            repository.list_snapshots_in_realm(Realm::China).unwrap()[0].entries[0].quantity,
            14
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejecting_a_candidate_removes_it_without_creating_a_snapshot() {
        let path = std::env::temp_dir().join(format!(
            "poe2-candidate-reject-{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let state = AppState::open(&path).unwrap();
        state
            .save_capture_candidate(CaptureCandidate::new(
                Some(Realm::China),
                vec![CurrencyAmount::new("exalted", 12)],
                88,
            ))
            .unwrap();

        let candidate = state.capture_candidates().unwrap().pop().unwrap();
        state.reject_capture_candidate(candidate.id).unwrap();

        assert!(state.capture_candidates().unwrap().is_empty());
        assert_eq!(state.snapshot_count().unwrap(), 0);
        std::fs::remove_file(path).unwrap();
    }
}

#[cfg(test)]
mod app_state_tests {
    use super::{
        commands::{CreateSnapshotInput, CurrencyQuantityInput},
        providers::{ProviderAvailability, ProviderId},
        realm::Realm,
        AppState,
    };

    #[test]
    fn app_state_saves_a_snapshot_to_its_local_database() {
        let path = std::env::temp_dir().join(format!("poe2-state-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let state = AppState::open(&path).unwrap();

        state
            .save_snapshot(CreateSnapshotInput {
                captured_at: "2026-09-03T09:00:00+08:00".into(),
                entries: vec![CurrencyQuantityInput {
                    currency_id: "exalted".into(),
                    quantity: 5,
                }],
            })
            .unwrap();

        assert_eq!(state.snapshot_count().unwrap(), 1);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn app_state_exposes_the_selected_realm() {
        let path =
            std::env::temp_dir().join(format!("poe2-state-realm-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let state = AppState::open(&path).unwrap();

        assert_eq!(state.realm().unwrap(), Realm::China);
        state.set_realm(Realm::China).unwrap();
        assert_eq!(state.realm().unwrap(), Realm::China);

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn app_state_reports_the_current_realms_price_provider_status() {
        let path = std::env::temp_dir().join(format!(
            "poe2-provider-status-{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let state = AppState::open(&path).unwrap();
        state.set_realm(Realm::China).unwrap();

        let status = state.price_provider_status().unwrap();

        assert_eq!(status.provider, ProviderId::CNMarket);
        assert_eq!(status.availability, ProviderAvailability::Unavailable);
        std::fs::remove_file(path).unwrap();
    }
}

#[cfg(test)]
mod daily_ledger_command_tests {
    use super::{
        commands::{CreateSnapshotInput, CurrencyQuantityInput},
        domain::{AdjustmentKind, Direction, LedgerAdjustment},
        AppState,
    };

    #[test]
    fn returns_the_daily_ledger_from_persisted_snapshots() {
        let path = std::env::temp_dir().join(format!("poe2-ledger-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let state = AppState::open(&path).unwrap();
        state
            .save_snapshot(CreateSnapshotInput {
                captured_at: "2026-09-03T09:00:00+08:00".into(),
                entries: vec![CurrencyQuantityInput {
                    currency_id: "exalted".into(),
                    quantity: 10,
                }],
            })
            .unwrap();
        state
            .save_snapshot(CreateSnapshotInput {
                captured_at: "2026-09-03T21:00:00+08:00".into(),
                entries: vec![CurrencyQuantityInput {
                    currency_id: "exalted".into(),
                    quantity: 17,
                }],
            })
            .unwrap();

        let ledger = state.daily_ledger("2026-09-03").unwrap();

        assert_eq!(ledger[0].net_change, 7);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn includes_persisted_adjustments_in_the_daily_ledger() {
        let path = std::env::temp_dir().join(format!(
            "poe2-ledger-adjustment-{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let state = AppState::open(&path).unwrap();
        state
            .save_snapshot(CreateSnapshotInput {
                captured_at: "2026-09-03T09:00:00+08:00".into(),
                entries: vec![CurrencyQuantityInput {
                    currency_id: "exalted".into(),
                    quantity: 10,
                }],
            })
            .unwrap();
        state
            .save_snapshot(CreateSnapshotInput {
                captured_at: "2026-09-03T21:00:00+08:00".into(),
                entries: vec![CurrencyQuantityInput {
                    currency_id: "exalted".into(),
                    quantity: 15,
                }],
            })
            .unwrap();
        state
            .save_adjustment(LedgerAdjustment::new(
                "2026-09-03T12:00:00+08:00",
                "exalted",
                2,
                Direction::Outflow,
                AdjustmentKind::Crafting,
            ))
            .unwrap();

        let ledger = state.daily_ledger("2026-09-03").unwrap();

        assert_eq!(ledger[0].explained_change, -2);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn returns_weekly_ledger_from_persisted_snapshots() {
        let path =
            std::env::temp_dir().join(format!("poe2-weekly-ledger-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let state = AppState::open(&path).unwrap();
        state
            .save_snapshot(CreateSnapshotInput {
                captured_at: "2026-09-01T09:00:00+08:00".into(),
                entries: vec![CurrencyQuantityInput {
                    currency_id: "exalted".into(),
                    quantity: 10,
                }],
            })
            .unwrap();
        state
            .save_snapshot(CreateSnapshotInput {
                captured_at: "2026-09-07T21:00:00+08:00".into(),
                entries: vec![CurrencyQuantityInput {
                    currency_id: "exalted".into(),
                    quantity: 16,
                }],
            })
            .unwrap();

        assert_eq!(state.weekly_ledger("2026-09-01").unwrap()[0].net_change, 6);
        std::fs::remove_file(path).unwrap();
    }
}

#[cfg(test)]
mod adjustment_storage_tests {
    use super::{
        domain::{
            calculate_day, AdjustmentKind, CurrencyAmount, Direction, LedgerAdjustment, Snapshot,
        },
        realm::Realm,
        storage::SqliteRepository,
    };

    #[test]
    fn persists_realm_adjustments_for_the_daily_ledger() {
        let path =
            std::env::temp_dir().join(format!("poe2-adjustments-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let repository = SqliteRepository::open(&path).unwrap();
        repository
            .save_snapshot_in_realm(
                Realm::China,
                &Snapshot::valid(
                    "2026-09-03T09:00:00+08:00",
                    vec![CurrencyAmount::new("exalted", 10)],
                ),
            )
            .unwrap();
        repository
            .save_snapshot_in_realm(
                Realm::China,
                &Snapshot::valid(
                    "2026-09-03T21:00:00+08:00",
                    vec![CurrencyAmount::new("exalted", 15)],
                ),
            )
            .unwrap();
        repository
            .save_adjustment_in_realm(
                Realm::China,
                &LedgerAdjustment::new(
                    "2026-09-03T12:00:00+08:00",
                    "exalted",
                    2,
                    Direction::Outflow,
                    AdjustmentKind::Crafting,
                ),
            )
            .unwrap();

        let rows = calculate_day(
            &repository.list_snapshots_in_realm(Realm::China).unwrap(),
            &repository.list_adjustments_in_realm(Realm::China).unwrap(),
            "2026-09-03",
        )
        .unwrap();

        assert_eq!(rows[0].net_change, 5);
        assert_eq!(rows[0].explained_change, -2);
        assert_eq!(rows[0].unattributed_change, 7);
        std::fs::remove_file(path).unwrap();
    }
}

#[cfg(test)]
mod realm_tests {
    use super::{change_realm, current_realm, realm::Realm, storage::SqliteRepository, AppState};

    #[test]
    fn parses_the_two_supported_realms() {
        assert_eq!(Realm::parse("international"), Some(Realm::International));
        assert_eq!(Realm::parse("china"), Some(Realm::China));
        assert_eq!(Realm::parse("other"), None);
    }

    #[test]
    fn serializes_realms_for_the_frontend_in_the_same_format_as_commands() {
        assert_eq!(serde_json::to_string(&Realm::China).unwrap(), "\"china\"");
        assert_eq!(serde_json::to_string(&Realm::International).unwrap(), "\"international\"");
    }

    #[test]
    fn persists_the_selected_realm_in_local_profile() {
        let path = std::env::temp_dir().join(format!("poe2-realm-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let repository = SqliteRepository::open(&path).unwrap();
        assert_eq!(repository.realm().unwrap(), Realm::China);
        repository.set_realm(Realm::China).unwrap();
        drop(repository);

        assert_eq!(
            SqliteRepository::open(&path).unwrap().realm().unwrap(),
            Realm::China
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn keeps_snapshots_isolated_by_realm() {
        let path = std::env::temp_dir().join(format!(
            "poe2-realm-snapshots-{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let repository = SqliteRepository::open(&path).unwrap();

        repository
            .save_snapshot_in_realm(
                Realm::International,
                &super::domain::Snapshot::valid(
                    "2026-09-03T09:00:00+08:00",
                    vec![super::domain::CurrencyAmount::new("exalted", 10)],
                ),
            )
            .unwrap();
        repository
            .save_snapshot_in_realm(
                Realm::China,
                &super::domain::Snapshot::valid(
                    "2026-09-03T09:00:00+08:00",
                    vec![super::domain::CurrencyAmount::new("exalted", 30)],
                ),
            )
            .unwrap();

        assert_eq!(
            repository
                .list_snapshots_in_realm(Realm::International)
                .unwrap()[0]
                .entries[0]
                .quantity,
            10
        );
        assert_eq!(
            repository.list_snapshots_in_realm(Realm::China).unwrap()[0].entries[0].quantity,
            30
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn changes_realm_through_the_command_boundary() {
        let path =
            std::env::temp_dir().join(format!("poe2-realm-command-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let state = AppState::open(&path).unwrap();

        change_realm(&state, "china").unwrap();

        assert_eq!(current_realm(&state).unwrap(), Realm::China);
        assert_eq!(
            change_realm(&state, "unsupported").unwrap_err().code,
            "invalid_realm"
        );
        std::fs::remove_file(path).unwrap();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_directory = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_directory)?;
            app.manage(AppState::open(&data_directory.join("ledger.sqlite"))?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_snapshot,
            create_adjustment,
            get_daily_ledger,
            get_weekly_ledger,
            get_realm,
            set_realm,
            get_price_provider_status,
            import_manual_prices,
            get_capture_candidates,
            confirm_capture_candidate,
            reject_capture_candidate
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
