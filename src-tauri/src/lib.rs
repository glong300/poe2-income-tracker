// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::Manager;

pub mod domain;
pub mod storage;
pub mod commands;

pub struct AppState {
    repository: std::sync::Mutex<storage::SqliteRepository>,
}

impl AppState {
    pub fn open(path: &std::path::Path) -> rusqlite::Result<Self> {
        Ok(Self { repository: std::sync::Mutex::new(storage::SqliteRepository::open(path)?) })
    }

    pub fn save_snapshot(&self, input: commands::CreateSnapshotInput) -> Result<(), commands::CommandError> {
        let repository = self.repository.lock().map_err(|_| commands::CommandError { code: "state_error", message: "本地账本不可用".into() })?;
        commands::create_snapshot(&repository, input)
    }

    pub fn snapshot_count(&self) -> rusqlite::Result<usize> {
        Ok(self.repository.lock().expect("repository lock is not poisoned").list_snapshots()?.len())
    }
}

#[tauri::command]
fn create_snapshot(
    state: tauri::State<'_, AppState>,
    input: commands::CreateSnapshotInput,
) -> Result<(), commands::CommandError> {
    state.save_snapshot(input)
}

#[cfg(test)]
mod ledger_tests {
    use super::domain::{
        calculate_day, AdjustmentKind, CurrencyAmount, Direction, LedgerAdjustment, Snapshot,
    };

    #[test]
    fn calculates_net_explained_and_unattributed_change_for_one_currency() {
        let snapshots = vec![
            Snapshot::valid("2026-09-03T09:00:00+08:00", vec![CurrencyAmount::new("exalted", 10)]),
            Snapshot::valid("2026-09-03T21:00:00+08:00", vec![CurrencyAmount::new("exalted", 17)]),
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
            Snapshot::valid("2026-09-03T09:00:00+08:00", vec![CurrencyAmount::new("exalted", 10)]),
            Snapshot::invalid("2026-09-03T12:00:00+08:00", vec![CurrencyAmount::new("exalted", 999)]),
            Snapshot::valid("2026-09-03T21:00:00+08:00", vec![CurrencyAmount::new("exalted", 17)]),
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
}

#[cfg(test)]
mod sqlite_repository_tests {
    use super::domain::{calculate_day, CurrencyAmount, Snapshot};
    use super::storage::SqliteRepository;

    #[test]
    fn persists_snapshots_and_rebuilds_daily_ledger_after_reopening() {
        let database_path = std::env::temp_dir().join(format!(
            "poe2-income-tracker-{}.sqlite",
            std::process::id()
        ));
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
        create_snapshot, validate_snapshot_input, CreateSnapshotInput, CurrencyQuantityInput,
    };
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
        let database_path = std::env::temp_dir().join(format!(
            "poe2-income-command-{}.sqlite",
            std::process::id()
        ));
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
}

#[cfg(test)]
mod app_state_tests {
    use super::{commands::{CreateSnapshotInput, CurrencyQuantityInput}, AppState};

    #[test]
    fn app_state_saves_a_snapshot_to_its_local_database() {
        let path = std::env::temp_dir().join(format!("poe2-state-{}.sqlite", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let state = AppState::open(&path).unwrap();

        state.save_snapshot(CreateSnapshotInput { captured_at: "2026-09-03T09:00:00+08:00".into(), entries: vec![CurrencyQuantityInput { currency_id: "exalted".into(), quantity: 5 }] }).unwrap();

        assert_eq!(state.snapshot_count().unwrap(), 1);
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
        .invoke_handler(tauri::generate_handler![create_snapshot])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
