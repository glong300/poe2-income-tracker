<script setup lang="ts">
import { ref } from "vue";
import SnapshotForm from "./components/SnapshotForm.vue";

const pendingEntryCount = ref<number | null>(null);

function recordPendingSnapshot(entries: { currencyId: string; quantity: number }[]) {
  pendingEntryCount.value = entries.length;
}
</script>

<template>
  <main class="app-shell">
    <section>
      <p class="eyebrow">LOCAL LEDGER / POE2</p>
      <h1>POE2 每日通货收益</h1>
      <p v-if="pendingEntryCount !== null" class="pending-summary">
        已记录 {{ pendingEntryCount }} 项通货余额，等待写入本地账本。
      </p>
    </section>
    <SnapshotForm
      :currencies="[{ id: 'exalted', name: '崇高石' }]"
      @submit="recordPendingSnapshot"
    />
  </main>
</template>

<style scoped>
.app-shell { min-height: 100vh; padding: 72px; display: grid; grid-template-columns: minmax(300px, 1fr) minmax(360px, 440px); gap: 64px; align-items: start; color: #f0eadc; background: #10100f; }
.eyebrow { color: #c78637; letter-spacing: .16em; font-size: 12px; }
h1 { max-width: 440px; margin: 12px 0; font: 700 46px Georgia, serif; line-height: 1.04; }
.pending-summary { color: #d8ccb7; }
</style>
