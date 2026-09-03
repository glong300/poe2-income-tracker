// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod domain;
pub mod storage;
pub mod commands;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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
    use super::commands::{validate_snapshot_input, CreateSnapshotInput, CurrencyQuantityInput};

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
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
