import { invoke } from "@tauri-apps/api/core";

export type SnapshotEntryInput = { currency_id: string; quantity: number };
export type DailyLedgerRow = { currency_id: string; net_change: number; explained_change: number; unattributed_change: number };
export type Realm = "international" | "china";
export type ProviderStatus = { provider: "PoeNinja" | "CNMarket"; availability: "AwaitingSync" | "Unavailable"; message: string };
export type CaptureCandidate = { id: number; candidate: { realm_hint: Realm | null; entries: SnapshotEntryInput[]; confidence: number } };
export type AdjustmentInput = { currency_id: string; quantity: number; direction: "inflow" | "outflow"; kind: "trade" | "exchange" | "crafting" | "other"; occurred_at: string };

export function saveSnapshot(entries: SnapshotEntryInput[]) {
  return invoke<void>("create_snapshot", {
    input: {
      captured_at: new Date().toISOString(),
      entries,
    },
  });
}
export function createAdjustment(input: AdjustmentInput) { return invoke<void>("create_adjustment", { input }); }

export function getDailyLedger(day: string) {
  return invoke<DailyLedgerRow[]>("get_daily_ledger", { day });
}
export function getWeeklyLedger(weekStart: string) {
  return invoke<DailyLedgerRow[]>("get_weekly_ledger", { weekStart });
}

export function getRealm() { return invoke<Realm>("get_realm"); }
export function setRealm(realm: Realm) { return invoke<void>("set_realm", { realm }); }
export function getPriceProviderStatus() { return invoke<ProviderStatus>("get_price_provider_status"); }
export function importManualPrices(csv: string) { return invoke<number>("import_manual_prices", { csv }); }
export function getCaptureCandidates() { return invoke<CaptureCandidate[]>("get_capture_candidates"); }
export function confirmCaptureCandidate(candidateId: number, entries: SnapshotEntryInput[]) {
  return invoke<void>("confirm_capture_candidate", {
    candidateId,
    capturedAt: new Date().toISOString(),
    entries,
  });
}
export function rejectCaptureCandidate(candidateId: number) { return invoke<void>("reject_capture_candidate", { candidateId }); }
