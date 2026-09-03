# POE2 每日通货收益追踪器 MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 构建一个 Windows 本地优先的桌面 MVP，允许玩家保存通货快照、记录收支调整，并查看每日通货账本。

**Architecture:** 使用 Tauri 2 的 Rust 后端保存 SQLite 数据并执行纯领域计算；Vue 3 前端经由显式 Tauri command 获取 DTO、提交用户表单。快照和调整记录是事实来源，日账本在读取时由有效快照和调整记录派生。

**Tech Stack:** Tauri 2、Rust、SQLx + SQLite、Vue 3、TypeScript、Vite、Vitest、Vue Test Utils。

**Spec:** `docs/superpowers/specs/2026-09-03-poe2-income-tracker-design.md`

## Global Constraints

- 仅支持 Windows；所有业务数据保存到本地 SQLite。
- 不增加网络请求、账号登录、云同步、游戏内存读取、客户端注入或自动输入。
- 截图/OCR 和 `Client.txt` 只定义 Rust trait，不读取文件或执行识别。
- 所有数量均为非负整数；调整记录的数量必须为正整数。
- 生产业务代码必须在对应失败测试被观察到后才实现。

---

## File Structure

- `src/`：Vue 应用与页面、组件、Tauri API 客户端。
- `src/lib/types.ts`：前端共享 DTO 类型。
- `src/lib/commands.ts`：唯一可调用 Tauri commands 的前端模块。
- `src/components/`：快照、调整和账本 UI 组件。
- `src/pages/`：今日、快照、通货、历史页面。
- `src-tauri/src/domain/`：无 I/O 的通货、快照、调整和账本规则。
- `src-tauri/src/storage/`：数据库迁移和 SQLite repository。
- `src-tauri/src/app/`：连接 repository 与领域服务的 application service。
- `src-tauri/src/commands/`：输入校验与 Tauri command DTO 映射。
- `src-tauri/src/adapters/`：未来 OCR 和游戏日志采集的空 trait 定义。
- `src-tauri/migrations/`：SQLite schema。
- `src-tauri/tests/`：数据库集成测试。
- `src/**/*.test.ts`：Vue 与前端工具测试。

## Task 1: Scaffold the desktop shell and test harness

**Files:**
- Create: `package.json`, `vite.config.ts`, `src/main.ts`, `src/App.vue`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`
- Create: `src/test/setup.ts`, `src/App.test.ts`
- Modify: `.gitignore`

**Interfaces:**
- Produces: a Tauri 2 + Vue 3 application runnable with `pnpm tauri dev`.
- Produces: `pnpm test` for browser-like component tests and `cargo test` for Rust tests.

- [ ] **Step 1: Create the application scaffold**

Run:

```bash
pnpm create tauri-app@latest . --template vue-ts --manager pnpm --identifier com.poe2tracker.app
pnpm add -D vitest @vue/test-utils jsdom
```

Choose the Tauri 2 Vue TypeScript template, with no bundled updater or network capability.

- [ ] **Step 2: Configure a failing root-component test**

Create `src/App.test.ts`:

```ts
import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import App from './App.vue';

describe('App', () => {
  it('renders the POE2 income tracker title', () => {
    expect(mount(App).get('h1').text()).toBe('POE2 每日通货收益');
  });
});
```

- [ ] **Step 3: Run the test to verify it fails**

Run: `pnpm test --run src/App.test.ts`

Expected: FAIL because the scaffold does not render `POE2 每日通货收益`.

- [ ] **Step 4: Implement the minimal application shell**

Replace `src/App.vue` content with:

```vue
<template>
  <main>
    <h1>POE2 每日通货收益</h1>
  </main>
</template>
```

Configure Vitest in `vite.config.ts` with `environment: 'jsdom'` and `setupFiles: ['./src/test/setup.ts']`; import `@testing-library/jest-dom/vitest` in the setup file if that dependency is chosen.

- [ ] **Step 5: Run the tests to verify the shell**

Run:

```bash
pnpm test --run
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: PASS with no failing tests.

- [ ] **Step 6: Commit the scaffold**

```bash
git add .gitignore package.json pnpm-lock.yaml vite.config.ts src src-tauri
git commit -m "feat: scaffold Tauri Vue desktop shell"
```

## Task 2: Implement and test the Rust ledger domain

**Files:**
- Create: `src-tauri/src/domain/mod.rs`
- Create: `src-tauri/src/domain/currency.rs`
- Create: `src-tauri/src/domain/snapshot.rs`
- Create: `src-tauri/src/domain/adjustment.rs`
- Create: `src-tauri/src/domain/ledger.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Produces: `calculate_day(entries: &[Snapshot], adjustments: &[LedgerAdjustment], date: NaiveDate) -> Vec<CurrencyDayLedger>`.
- Produces: `Snapshot::new(captured_at, entries) -> Result<Snapshot, DomainError>`.
- Produces: `LedgerAdjustment::new(occurred_at, currency_id, quantity, direction, kind) -> Result<LedgerAdjustment, DomainError>`.

- [ ] **Step 1: Write the failing daily-ledger tests**

Create tests inside `src-tauri/src/domain/ledger.rs`:

```rust
#[test]
fn calculates_net_explained_and_unattributed_change_for_one_currency() {
    let day = date(2026, 9, 3);
    let snapshots = vec![snapshot(day, 9, 10), snapshot(day, 21, 17)];
    let adjustments = vec![adjustment(day, 12, 4, Direction::Inflow)];

    let rows = calculate_day(&snapshots, &adjustments, day);

    assert_eq!(rows, vec![CurrencyDayLedger::new("exalted", 7, 4, 3)]);
}

#[test]
fn excludes_invalid_snapshots_when_calculating_a_day() {
    let day = date(2026, 9, 3);
    let snapshots = vec![snapshot(day, 9, 10), invalid_snapshot(day, 12, 999), snapshot(day, 21, 17)];

    assert_eq!(calculate_day(&snapshots, &[], day)[0].net_change, 7);
}
```

- [ ] **Step 2: Run the Rust test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml domain::ledger`

Expected: FAIL because the domain types and `calculate_day` do not exist.

- [ ] **Step 3: Implement the smallest domain model and calculation**

Implement immutable `Snapshot` values with `captured_at`, `status`, and a `BTreeMap<CurrencyId, u64>`; define `Direction`, `AdjustmentKind`, `LedgerAdjustment`, and `CurrencyDayLedger`. Filter by date and `SnapshotStatus::Valid`, choose the earliest and latest snapshot per currency, and sum adjustment signs by currency.

- [ ] **Step 4: Add validation tests before validation implementation**

Add these tests:

```rust
#[test]
fn rejects_duplicate_currency_entries_in_a_snapshot() {
    assert!(Snapshot::new(now(), vec![("exalted", 1), ("exalted", 2)]).is_err());
}

#[test]
fn returns_no_ledger_when_a_currency_has_only_one_valid_snapshot() {
    assert!(calculate_day(&vec![snapshot(date(2026, 9, 3), 9, 10)], &[], date(2026, 9, 3)).is_empty());
}
```

- [ ] **Step 5: Run the validation tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml domain::ledger`

Expected: FAIL until duplicate validation and single-snapshot exclusion are implemented.

- [ ] **Step 6: Implement validation and re-run all Rust domain tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml domain`

Expected: PASS.

- [ ] **Step 7: Commit the domain layer**

```bash
git add src-tauri/src/domain src-tauri/src/lib.rs
git commit -m "feat: add currency ledger domain"
```

## Task 3: Persist facts in SQLite and expose application services

**Files:**
- Create: `src-tauri/migrations/0001_initial.sql`
- Create: `src-tauri/src/storage/mod.rs`
- Create: `src-tauri/src/storage/sqlite_repository.rs`
- Create: `src-tauri/src/app/mod.rs`
- Create: `src-tauri/src/app/ledger_service.rs`
- Create: `src-tauri/tests/sqlite_repository.rs`
- Modify: `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`

**Interfaces:**
- Produces: `SqliteRepository::open(path: &Path) -> Result<Self, StorageError>`.
- Produces: `save_snapshot(&self, snapshot: &Snapshot) -> Result<(), StorageError>`.
- Produces: `save_adjustment(&self, adjustment: &LedgerAdjustment) -> Result<(), StorageError>`.
- Produces: `LedgerService::daily_ledger(&self, date: NaiveDate) -> Result<Vec<CurrencyDayLedger>, AppError>`.

- [ ] **Step 1: Write the failing temporary-database persistence test**

Create `src-tauri/tests/sqlite_repository.rs`:

```rust
#[test]
fn persists_snapshots_and_rebuilds_the_day_ledger_after_reopening() {
    let temp = tempfile::NamedTempFile::new().unwrap();
    let repository = SqliteRepository::open(temp.path()).unwrap();
    repository.save_snapshot(&snapshot(date(2026, 9, 3), 9, 10)).unwrap();
    repository.save_snapshot(&snapshot(date(2026, 9, 3), 21, 17)).unwrap();
    drop(repository);

    let service = LedgerService::open(temp.path()).unwrap();
    assert_eq!(service.daily_ledger(date(2026, 9, 3)).unwrap()[0].net_change, 7);
}
```

- [ ] **Step 2: Run the integration test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test sqlite_repository`

Expected: FAIL because repository and service are missing.

- [ ] **Step 3: Add the first migration and minimal repository**

Create tables for `currency_definitions`, `snapshots`, `snapshot_entries`, and `ledger_adjustments`. Add foreign keys, a unique `(snapshot_id, currency_id)` constraint, and a `CHECK(quantity >= 0)` constraint. Run migrations when the repository opens.

- [ ] **Step 4: Implement `LedgerService` and make the test pass**

`LedgerService` loads persisted snapshots and adjustments for the requested date, then delegates calculation to `domain::ledger::calculate_day`; it must not duplicate calculation rules in SQL.

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test sqlite_repository`

Expected: PASS.

- [ ] **Step 5: Add a failing invalid-snapshot test**

```rust
#[test]
fn invalidating_a_snapshot_removes_it_from_rebuilt_ledger() {
    let service = service_with_two_snapshots();
    service.invalidate_snapshot(snapshot_id_at_noon()).unwrap();

    assert!(service.daily_ledger(date(2026, 9, 3)).unwrap().is_empty());
}
```

- [ ] **Step 6: Implement invalidation and verify all Rust tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: PASS.

- [ ] **Step 7: Commit persistence and services**

```bash
git add src-tauri/Cargo.toml src-tauri/migrations src-tauri/src src-tauri/tests
git commit -m "feat: persist local snapshots and adjustments"
```

## Task 4: Define safe Tauri commands and future capture boundaries

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/ledger_commands.rs`
- Create: `src-tauri/src/adapters/mod.rs`
- Create: `src-tauri/src/adapters/capture.rs`
- Create: `src-tauri/src/adapters/game_log.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Produces: Tauri commands `list_currencies`, `create_snapshot`, `create_adjustment`, `get_daily_ledger`, and `invalidate_snapshot`.
- Produces: `CaptureCandidateSource` and `SessionEventSource` traits with no concrete filesystem implementation.

- [ ] **Step 1: Write a failing command-input validation test**

```rust
#[test]
fn create_snapshot_command_rejects_a_negative_quantity() {
    let result = validate_snapshot_input(CreateSnapshotInput {
        captured_at: "2026-09-03T09:00:00+08:00".into(),
        entries: vec![CurrencyQuantityInput { currency_id: "exalted".into(), quantity: -1 }],
        note: None,
    });

    assert_eq!(result.unwrap_err().code, "invalid_quantity");
}
```

- [ ] **Step 2: Run the command test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml commands`

Expected: FAIL because command DTOs and validator do not exist.

- [ ] **Step 3: Implement DTO conversion and commands**

Keep `quantity` as a signed number only at the command boundary; validate it then convert to `u64` before invoking `LedgerService`. Return a typed `CommandError { code, message }`; do not expose SQLite error strings to the frontend.

- [ ] **Step 4: Define the no-I/O extension traits**

```rust
pub trait CaptureCandidateSource {
    fn candidates(&self) -> Result<Vec<CaptureCandidate>, AdapterError>;
}

pub trait SessionEventSource {
    fn recent_events(&self) -> Result<Vec<SessionEvent>, AdapterError>;
}
```

Do not create a screenshot reader, OCR invocation, `Client.txt` path, file watcher, network client, or implementation of either trait.

- [ ] **Step 5: Re-run Rust tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: PASS.

- [ ] **Step 6: Commit the command boundary**

```bash
git add src-tauri/src
git commit -m "feat: expose validated local ledger commands"
```

## Task 5: Build the four Vue workflows

**Files:**
- Create: `src/lib/types.ts`
- Create: `src/lib/commands.ts`
- Create: `src/pages/TodayPage.vue`, `src/pages/SnapshotsPage.vue`, `src/pages/CurrenciesPage.vue`, `src/pages/HistoryPage.vue`
- Create: `src/components/SnapshotForm.vue`, `src/components/AdjustmentForm.vue`, `src/components/LedgerTable.vue`, `src/components/AppNavigation.vue`
- Create: `src/components/SnapshotForm.test.ts`, `src/components/LedgerTable.test.ts`
- Modify: `src/App.vue`, `src/main.ts`

**Interfaces:**
- Produces: `createSnapshot(input: CreateSnapshotInput): Promise<void>` and `getDailyLedger(date: string): Promise<CurrencyDayLedger[]>` in `src/lib/commands.ts`.
- Produces: `SnapshotForm` emitting `submit` only with valid, nonnegative quantities.
- Produces: `LedgerTable` rendering net, explained, and unattributed values for a currency.

- [ ] **Step 1: Write the failing snapshot-form validation test**

```ts
it('does not submit a negative currency quantity', async () => {
  const wrapper = mount(SnapshotForm, { props: { currencies: [{ id: 'exalted', name: '崇高石' }] } });
  await wrapper.get('[name="quantity-exalted"]').setValue('-1');
  await wrapper.get('form').trigger('submit');

  expect(wrapper.emitted('submit')).toBeUndefined();
  expect(wrapper.text()).toContain('数量必须是非负整数');
});
```

- [ ] **Step 2: Run the component test to verify it fails**

Run: `pnpm test --run src/components/SnapshotForm.test.ts`

Expected: FAIL because `SnapshotForm` does not exist.

- [ ] **Step 3: Implement the snapshot form and Tauri API wrapper**

`src/lib/commands.ts` is the only front-end file importing `invoke` from `@tauri-apps/api/core`. Implement client-side integer validation for immediate feedback, then pass valid payloads to `create_snapshot`.

- [ ] **Step 4: Write and run the failing ledger-table test**

```ts
it('renders all three ledger measures for a currency', () => {
  const wrapper = mount(LedgerTable, {
    props: { rows: [{ currencyId: 'exalted', currencyName: '崇高石', netChange: 7, explainedChange: 4, unattributedChange: 3 }] },
  });

  expect(wrapper.text()).toContain('崇高石');
  expect(wrapper.text()).toContain('净变化 7');
  expect(wrapper.text()).toContain('已解释 4');
  expect(wrapper.text()).toContain('未归因 3');
});
```

Run: `pnpm test --run src/components/LedgerTable.test.ts`

Expected: FAIL because `LedgerTable` does not exist.

- [ ] **Step 5: Implement the table, navigation and pages**

Wire pages through a small local `ref<'today' | 'snapshots' | 'currencies' | 'history'>` in `App.vue`; do not introduce a router dependency. Today calls `getDailyLedger` for the local date. Snapshots renders `SnapshotForm`. Currencies renders `AdjustmentForm`. History requests past daily ledgers. Render a clear message only when a query completed and a date has no computable ledger.

- [ ] **Step 6: Run all frontend and backend tests**

Run:

```bash
pnpm test --run
cargo test --manifest-path src-tauri/Cargo.toml
pnpm tauri build
```

Expected: all tests pass and the Windows bundle builds successfully.

- [ ] **Step 7: Commit the MVP UI**

```bash
git add src
git commit -m "feat: add local currency ledger interface"
```

## Task 6: Document local operation and verify a clean checkout

**Files:**
- Create: `README.md`
- Modify: `.gitignore`

**Interfaces:**
- Produces: exact setup, test and run commands for a Windows developer.

- [ ] **Step 1: Write README acceptance assertions as a checklist**

Add this exact checklist to `README.md`:

```markdown
- [ ] Create two valid snapshots on the same date for one currency.
- [ ] Add one inflow adjustment for that currency.
- [ ] Verify Today shows net, explained, and unattributed changes.
- [ ] Mark the last snapshot invalid and verify that day's row disappears.
- [ ] Close and reopen the app; verify the saved facts remain visible.
```

- [ ] **Step 2: Verify the documented commands on a clean state**

Run:

```bash
pnpm install --frozen-lockfile
pnpm test --run
cargo test --manifest-path src-tauri/Cargo.toml
pnpm tauri build
git status --short
```

Expected: tests and build pass; `.gitignore` prevents local SQLite data and build outputs from appearing as untracked files.

- [ ] **Step 3: Commit the documentation**

```bash
git add README.md .gitignore
git commit -m "docs: add local MVP setup guide"
```
