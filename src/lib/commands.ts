import { invoke } from "@tauri-apps/api/core";

export type SnapshotEntryInput = { currency_id: string; quantity: number };

export function saveSnapshot(entries: SnapshotEntryInput[]) {
  return invoke<void>("create_snapshot", {
    input: {
      captured_at: new Date().toISOString(),
      entries,
    },
  });
}
