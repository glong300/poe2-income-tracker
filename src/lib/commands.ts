import { invoke } from "@tauri-apps/api/core";

export type SnapshotEntryInput = { currency_id: string; quantity: number };
export type DailyLedgerRow = { currency_id: string; net_change: number; explained_change: number; unattributed_change: number };
export type Realm = "international" | "china";
export type ProviderStatus = { provider: "PoeNinja" | "CNMarket"; availability: "AwaitingSync" | "Unavailable"; message: string };

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

export function getRealm() { return invoke<Realm>("get_realm"); }
export function setRealm(realm: Realm) { return invoke<void>("set_realm", { realm }); }
export function getPriceProviderStatus() { return invoke<ProviderStatus>("get_price_provider_status"); }
