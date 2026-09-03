<script setup lang="ts">
import { onMounted, ref } from "vue";
import SnapshotForm from "./components/SnapshotForm.vue";
import { getDailyLedger, saveSnapshot, type DailyLedgerRow } from "./lib/commands";

const savedEntryCount = ref<number | null>(null);
const saveError = ref("");
const ledgerRows = ref<DailyLedgerRow[]>([]);

async function recordSnapshot(entries: { currencyId: string; quantity: number }[]) {
  saveError.value = "";
  try {
    await saveSnapshot(entries.map((entry) => ({ currency_id: entry.currencyId, quantity: entry.quantity })));
    savedEntryCount.value = entries.length;
  } catch {
    saveError.value = "无法保存本地快照，请重试。";
  }
}

onMounted(async () => {
  ledgerRows.value = await getDailyLedger(new Date().toISOString().slice(0, 10));
});
</script>

<template>
  <main class="app-shell">
    <section>
      <p class="eyebrow">LOCAL LEDGER / POE2</p>
      <h1>POE2 每日通货收益</h1>
      <p v-if="savedEntryCount !== null" class="pending-summary">
        快照已保存：{{ savedEntryCount }} 项通货余额。
      </p>
      <p v-if="saveError" class="save-error" role="alert">{{ saveError }}</p>
      <div class="ledger-summary">
        <span>今日净变化</span>
        <strong>{{ ledgerRows.reduce((total, row) => total + row.net_change, 0) }}</strong>
      </div>
    </section>
    <SnapshotForm
      :currencies="[{ id: 'exalted', name: '崇高石' }]"
      @submit="recordSnapshot"
    />
  </main>
</template>

<style scoped>
.app-shell { min-height: 100vh; padding: 72px; display: grid; grid-template-columns: minmax(300px, 1fr) minmax(360px, 440px); gap: 64px; align-items: start; color: #f0eadc; background: #10100f; }
.eyebrow { color: #c78637; letter-spacing: .16em; font-size: 12px; }
h1 { max-width: 440px; margin: 12px 0; font: 700 46px Georgia, serif; line-height: 1.04; }
.pending-summary { color: #d8ccb7; }
.save-error { color: #ff9b78; }
</style>
