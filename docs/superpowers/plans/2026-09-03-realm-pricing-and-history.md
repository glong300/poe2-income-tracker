# POE2 区服、价格与收益历史 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将本地账本升级为国服/国际服隔离、可估值、可查看历史和可人工确认 OCR 候选的收益工具。

**Architecture:** 所有事实数据增加 `realm`，由本地资料决定当前经济区。价格采用 provider 适配器和手动确认快照两层，日账本只读取当前区服数据；Vue 通过 Tauri commands 展示摘要、历史和同步状态。

**Tech Stack:** Tauri 2、Rust、rusqlite、Vue 3、TypeScript、Vitest。

**Spec:** `docs/superpowers/specs/2026-09-03-realm-pricing-and-history-design.md`

## Global Constraints

- 仅支持 Windows 与本地 SQLite；不读取游戏内存、不自动输入、不上传用户数据。
- `international` 与 `china` 数据绝不混算。
- 手动确认价格优先于自动价格；自动价格失败显示状态，不阻断账本。
- OCR 和日志仅产生候选记录，用户确认后才写入快照。

---

### Task 1: Realm profile and data isolation

**Files:**
- Create: `src-tauri/src/realm.rs`
- Modify: `src-tauri/src/storage.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/commands.rs`
- Test: Rust tests in `src-tauri/src/lib.rs`

**Interfaces:**
- Produces `enum Realm { International, China }` and `Profile { realm: Realm, override_enabled: bool }`.
- Produces `AppState::set_realm(realm)` and `AppState::realm()`.

- [ ] Write a failing test that saves an `international` exalted snapshot and a `china` exalted snapshot, then asserts each realm ledger has only its own quantity.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml realm_tests`; expect failure because `Realm` and realm-filtered storage do not exist.
- [ ] Add `realm TEXT NOT NULL` columns and a one-row `profile` table; migrate existing rows to `international`.
- [ ] Implement `Realm`, profile persistence, realm-filtered repository queries and commands.
- [ ] Re-run `cargo test --manifest-path src-tauri/Cargo.toml`; expect pass; commit `feat: isolate data by realm`.

### Task 2: Manual and automatic price snapshots

**Files:**
- Create: `src-tauri/src/pricing.rs`, `src-tauri/src/providers/mod.rs`, `src-tauri/src/providers/poe_ninja.rs`, `src-tauri/src/providers/cn_market.rs`
- Modify: `src-tauri/src/storage.rs`, `src-tauri/src/commands.rs`
- Test: Rust provider and priority tests in `src-tauri/src/pricing.rs`

**Interfaces:**
- Produces `PriceSnapshot { realm, currency_id, value, quoted_in, source, captured_at, confirmed }`.
- Produces `effective_price(realm, currency_id, at) -> Option<PriceSnapshot>`.

- [ ] Write a failing test with automatic price `10` and confirmed manual price `12` for the same realm/currency/time; assert `effective_price` returns `12`.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml pricing`; expect failure.
- [ ] Create price tables keyed by realm, currency and capture time; persist source/update status.
- [ ] Implement `PoeNinjaProvider` for international only; implement `CNMarketProvider` as a no-network adapter returning provider-unavailable until a verified endpoint is configured.
- [ ] Add CSV parser accepting headers `currency_id,value,quoted_in,captured_at`; reject a row with an unknown realm or non-positive value.
- [ ] Run Rust tests; commit `feat: add realm-aware price snapshots`.

### Task 3: Ledger history and adjustments

**Files:**
- Modify: `src-tauri/src/domain.rs`, `src-tauri/src/storage.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`
- Test: Rust day/week history tests in `src-tauri/src/lib.rs`

**Interfaces:**
- Produces `get_daily_ledger(realm, day)` and `get_weekly_ledger(realm, week_start)`.
- Produces `create_adjustment(realm, currency_id, quantity, direction, kind, occurred_at)`.

- [ ] Write a failing test recording two snapshots and one crafting outflow in one realm; assert daily row reports net, explained and unattributed values.
- [ ] Run focused Cargo test; expect failure because adjustments are not persisted.
- [ ] Add `ledger_adjustments` table with realm, direction, kind, quantity and time.
- [ ] Implement repository and command methods; aggregate day/week only after realm filtering.
- [ ] Run all Rust tests; commit `feat: add realm-aware ledger history`.

### Task 4: Dashboard, history and price UI

**Files:**
- Create: `src/components/LedgerTable.vue`, `src/components/RealmSelector.vue`, `src/pages/HistoryPage.vue`, `src/pages/PricingPage.vue`
- Modify: `src/App.vue`, `src/lib/commands.ts`
- Test: `src/components/RealmSelector.test.ts`, `src/components/LedgerTable.test.ts`

**Interfaces:**
- `RealmSelector` emits `change` with `international | china`.
- `LedgerTable` consumes `{ currencyId, netChange, explainedChange, unattributedChange }[]`.

- [ ] Write a failing `RealmSelector` test: selecting 国服 emits `china` and current badge changes.
- [ ] Run `pnpm exec vitest run src/components/RealmSelector.test.ts`; expect failure.
- [ ] Implement selector, deep background reset, responsive one-column layout below 900px, summary metrics and ledger table.
- [ ] Write a failing table test asserting a negative unexplained amount is visually labelled `未归因`.
- [ ] Implement history date/week controls and pricing page with automatic/manual source, update time and failure state.
- [ ] Run `pnpm exec vitest run`; commit `feat: add realm-aware dashboard and history`.

### Task 5: OCR candidate review boundary

**Files:**
- Create: `src-tauri/src/adapters/capture.rs`, `src/components/CandidateReview.vue`
- Modify: `src-tauri/src/lib.rs`, `src-tauri/src/commands.rs`
- Test: Rust and Vue candidate-review tests

**Interfaces:**
- Produces `CaptureCandidate { realm_hint, entries, confidence }`.
- Produces `confirm_candidate(candidate_id, realm, entries)`; only this command creates a snapshot.

- [ ] Write a failing test that a candidate remains absent from `list_snapshots` until `confirm_candidate` is called.
- [ ] Run focused Cargo test; expect failure.
- [ ] Implement local candidate storage and confirm/reject commands, with no file watcher or OCR engine implementation.
- [ ] Implement review UI showing confidence, detected realm suggestion and editable quantities.
- [ ] Run all front-end and Rust tests; commit `feat: add manual capture candidate review`.

### Task 6: Documentation and release verification

**Files:**
- Modify: `README.md`

- [ ] Document realm selection, price source priority, CSV header format and local-only privacy boundary.
- [ ] Run `pnpm exec vitest run`, `cargo test --manifest-path src-tauri/Cargo.toml`, and `pnpm tauri build`.
- [ ] Verify a clean `git status --short`; commit `docs: document realm pricing workflow` and push the branch.
