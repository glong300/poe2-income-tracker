import { invoke } from "@tauri-apps/api/core";

export type SnapshotEntryInput = { currency_id: string; quantity: number };
export type DailyLedgerRow = { currency_id: string; net_change: number; explained_change: number; unattributed_change: number };

export function saveSnapshot(entries: SnapshotEntryInput[]) {
  return invoke<void>("create_snapshot", {
    input: {
      captured_at: new Date().toISOString(),
      entries,
    },
  });
}

export function getDailyLedger(day: string) {
  return invoke<DailyLedgerRow[]>("get_daily_ledger", { day });
}
