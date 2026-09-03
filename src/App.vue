<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import LedgerTable from "./components/LedgerTable.vue";
import AdjustmentForm from "./components/AdjustmentForm.vue";
import RealmSelector from "./components/RealmSelector.vue";
import SnapshotForm from "./components/SnapshotForm.vue";
import CandidateReview from "./components/CandidateReview.vue";
import HistoryPage from "./pages/HistoryPage.vue";
import PricingPage from "./pages/PricingPage.vue";
import { confirmCaptureCandidate, createAdjustment, getCaptureCandidates, getDailyLedger, getPriceProviderStatus, getRealm, getWeeklyLedger, importManualPrices, rejectCaptureCandidate, saveSnapshot, setRealm, type CaptureCandidate, type DailyLedgerRow, type ProviderStatus, type Realm } from "./lib/commands";

const savedEntryCount = ref<number | null>(null);
const saveError = ref("");
const ledgerRows = ref<DailyLedgerRow[]>([]);
const realm = ref<Realm>("china");
const providerStatus = ref<ProviderStatus | null>(null);
const isChangingRealm = ref(false);
const activeView = ref<"dashboard" | "history" | "pricing">("dashboard");
const historyRows = ref<DailyLedgerRow[]>([]);
const candidates = ref<CaptureCandidate[]>([]);
const netChange = computed(() => ledgerRows.value.reduce((total, row) => total + row.net_change, 0));
const unexplainedChange = computed(() => ledgerRows.value.reduce((total, row) => total + row.unattributed_change, 0));

async function refreshDashboard() {
  ledgerRows.value = await getDailyLedger(new Date().toISOString().slice(0, 10));
  providerStatus.value = await getPriceProviderStatus();
  candidates.value = await getCaptureCandidates();
}
async function recordSnapshot(entries: { currencyId: string; quantity: number }[]) {
  saveError.value = "";
  try { await saveSnapshot(entries.map((entry) => ({ currency_id: entry.currencyId, quantity: entry.quantity }))); savedEntryCount.value = entries.length; await refreshDashboard(); }
  catch { saveError.value = "无法保存本地快照，请重试。"; }
}
async function changeRealm(nextRealm: Realm) {
  if (nextRealm === realm.value || isChangingRealm.value) return;
  isChangingRealm.value = true;
  try { await setRealm(nextRealm); realm.value = nextRealm; await refreshDashboard(); }
  catch { saveError.value = "无法切换区服，请重试。"; }
  finally { isChangingRealm.value = false; }
}
async function loadDay(day: string) { historyRows.value = await getDailyLedger(day); }
async function loadWeek(weekStart: string) { historyRows.value = await getWeeklyLedger(weekStart); }
async function importPrices(csv: string) {
  saveError.value = "";
  try { const count = await importManualPrices(csv); savedEntryCount.value = count; await refreshDashboard(); }
  catch { saveError.value = "CSV 格式无效或无法保存本地价格。"; }
}
async function recordAdjustment(input: { currencyId: string; quantity: number; direction: "inflow" | "outflow"; kind: "trade" | "exchange" | "crafting" | "other" }) {
  saveError.value = "";
  try { await createAdjustment({ currency_id: input.currencyId, quantity: input.quantity, direction: input.direction, kind: input.kind, occurred_at: new Date().toISOString() }); await refreshDashboard(); }
  catch { saveError.value = "无法保存收支调整，请重试。"; }
}
async function confirmCandidate(id: number, entries: { currencyId: string; quantity: number }[]) {
  saveError.value = "";
  try { await confirmCaptureCandidate(id, entries.map((entry) => ({ currency_id: entry.currencyId, quantity: entry.quantity }))); await refreshDashboard(); }
  catch { saveError.value = "无法确认候选快照，请检查数量后重试。"; }
}
async function rejectCandidate(id: number) {
  try { await rejectCaptureCandidate(id); await refreshDashboard(); }
  catch { saveError.value = "无法移除候选记录。"; }
}
onMounted(async () => { try { realm.value = await getRealm(); await refreshDashboard(); } catch { saveError.value = "无法读取本地数据。"; } });
</script>

<template>
  <main class="app-shell">
    <header class="topbar"><div class="brand"><b>II</b><span>POE2 · 通货账本</span></div><RealmSelector :model-value="realm" @change="changeRealm" /></header>
    <section class="hero"><div><p class="eyebrow">LOCAL ECONOMY LEDGER</p><h1>每日收益，<em>清楚记账。</em></h1><p class="lead">区服、快照与价格记录全部保存在本机；每次变动都有迹可循。</p></div><div class="provider-status" :class="{ unavailable: providerStatus?.availability === 'Unavailable' }"><i aria-hidden="true"></i><div><strong>{{ providerStatus?.provider === "CNMarket" ? "国服行情" : "国际服行情" }}</strong><p>{{ providerStatus?.message ?? "正在读取行情状态" }}</p></div></div></section>
    <nav class="workspace-nav" aria-label="账本工作区"><button :class="{ active: activeView === 'dashboard' }" type="button" @click="activeView = 'dashboard'">今日账本</button><button :class="{ active: activeView === 'history' }" type="button" @click="activeView = 'history'">收益历史</button><button data-testid="nav-pricing" :class="{ active: activeView === 'pricing' }" type="button" @click="activeView = 'pricing'">价格管理</button></nav>
    <section v-if="activeView === 'dashboard'" class="dashboard-view"><section class="metrics" aria-label="今日账本摘要"><article><span>今日净变化</span><strong :class="{ positive: netChange > 0 }">{{ netChange > 0 ? `+${netChange}` : netChange }}</strong><small>通货单位</small></article><article><span>未归因变化</span><strong>{{ unexplainedChange > 0 ? `+${unexplainedChange}` : unexplainedChange }}</strong><small>需要补充收支记录</small></article><article><span>今日记录通货</span><strong>{{ ledgerRows.length }}</strong><small>来自有效快照</small></article></section><section class="workspace"><LedgerTable :rows="ledgerRows.map((row) => ({ currencyId: row.currency_id, netChange: row.net_change, explainedChange: row.explained_change, unattributedChange: row.unattributed_change }))" /><aside><SnapshotForm :currencies="[{ id: 'exalted', name: '崇高石' }]" @submit="recordSnapshot" /><AdjustmentForm :currencies="[{ id: 'exalted', name: '崇高石' }]" @submit="recordAdjustment" /><CandidateReview v-for="candidate in candidates" :key="candidate.id" :candidate="candidate" @confirm="confirmCandidate" @reject="rejectCandidate" /><p v-if="savedEntryCount !== null" class="notice">快照已保存：{{ savedEntryCount }} 项通货余额。</p><p v-if="saveError" class="error" role="alert">{{ saveError }}</p></aside></section></section>
    <HistoryPage v-else-if="activeView === 'history'" :rows="historyRows" @load-day="loadDay" @load-week="loadWeek" />
    <section v-else class="pricing-view"><PricingPage :provider-status="providerStatus" @import-csv="importPrices" /><p v-if="savedEntryCount !== null" class="notice">本次已写入 {{ savedEntryCount }} 项本地记录。</p><p v-if="saveError" class="error" role="alert">{{ saveError }}</p></section>
  </main>
</template>

<style>
:root { font-family: "Avenir Next", "Noto Sans SC", sans-serif; color: #f4efdf; background: #10110e; } * { box-sizing: border-box; } html, body, #app { min-width: 320px; min-height: 100%; margin: 0; } body { min-height: 100vh; background: #10110e; } button, input { font: inherit; }
</style>
<style scoped>
.app-shell { min-height: 100vh; padding: 24px clamp(20px,5vw,76px) 64px; background: radial-gradient(circle at 85% 0%,#302516 0,transparent 27rem),#10110e; }.topbar,.hero,.provider-status,.metrics,.workspace { display: flex; }.topbar { justify-content: space-between; align-items:center; padding-bottom:24px; border-bottom:1px solid #35342c; }.brand { display:flex; align-items:center; gap:10px; color:#e7dfce; font-size:14px; font-weight:700; }.brand b { display:grid; place-items:center; width:29px; height:29px; color:#1a140c; background:#d79d46; font-family:Georgia,serif; }.hero { justify-content:space-between; gap:30px; padding:clamp(48px,8vw,104px) 0 30px; }.eyebrow { margin:0; color:#d5a557; font-size:11px; font-weight:700; letter-spacing:.18em; }.hero h1 { margin:12px 0 14px; font:700 clamp(42px,6vw,76px)/.98 Georgia,serif; letter-spacing:-.04em; }.hero em,.positive { color:#d79d46; }.lead { max-width:540px; margin:0; color:#b8b0a0; line-height:1.7; }.provider-status { align-self:end; gap:12px; min-width:240px; padding:16px; background:#1a1b17; border:1px solid #3d3b31; border-radius:13px; }.provider-status i { width:9px; height:9px; margin-top:5px; border-radius:50%; background:#d79d46; box-shadow:0 0 0 4px #3b2b15; }.provider-status.unavailable i { background:#b58d65; }.provider-status p { margin:4px 0 0; color:#aaa394; font-size:13px; }.workspace-nav { display:flex; gap:8px; margin-bottom:18px; }.workspace-nav button { min-height:38px; padding:0 13px; color:#afa796; background:transparent; border:1px solid #423f35; border-radius:8px; cursor:pointer; }.workspace-nav button.active { color:#1b150d; font-weight:800; background:#d79d46; border-color:#d79d46; }.metrics { display:grid; grid-template-columns:repeat(3,1fr); gap:14px; }.metrics article,.notice,.error { padding:20px; background:#191a16; border:1px solid #3d3c33; border-radius:14px; }.metrics article { display:grid; gap:7px; }.metrics span,.metrics small { color:#a8a192; font-size:12px; }.metrics strong { font:700 31px Georgia,serif; }.workspace { display:grid; grid-template-columns:minmax(0,1fr) minmax(310px,380px); gap:18px; margin-top:18px; }.workspace aside,.pricing-view { display:grid; align-content:start; gap:10px; }.notice,.error { margin:0; padding:12px 14px; font-size:13px; }.notice { color:#d9e2bd; background:#293124; }.error { color:#ffd0b8; background:#44291f; } @media (max-width:900px) { .hero { display:grid; }.provider-status { align-self:start; }.workspace { grid-template-columns:1fr; } } @media (max-width:620px) { .app-shell { padding:18px 16px 40px; }.topbar { align-items:flex-start; gap:14px; flex-direction:column; }.metrics { grid-template-columns:1fr; }.hero h1 { font-size:48px; }.workspace-nav { overflow:auto; } } @media (prefers-reduced-motion:no-preference) { .metrics article { transition:transform 180ms ease,border-color 180ms ease; }.metrics article:hover { border-color:#7b663e; transform:translateY(-2px); } }
</style>
