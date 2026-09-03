<script setup lang="ts">
import { ref } from "vue";

defineProps<{ rows: { currency_id: string; net_change: number; explained_change: number; unattributed_change: number }[] }>();
const day = ref(new Date().toISOString().slice(0, 10));
const emit = defineEmits<{ "load-day": [day: string]; "load-week": [weekStart: string] }>();

function signed(value: number) { return value > 0 ? `+${value}` : String(value); }
</script>

<template>
  <section class="history-page" aria-labelledby="history-title">
    <header><div><p>HISTORY</p><h2 id="history-title">日 / 周收益历史</h2></div><input v-model="day" type="date" aria-label="历史日期" /></header>
    <div class="actions"><button type="button" @click="emit('load-day', day)">查看当天</button><button data-testid="weekly-history" type="button" class="secondary" @click="emit('load-week', day)">查看本周</button></div>
    <div v-if="rows.length" class="history-rows"><div v-for="row in rows" :key="row.currency_id" class="history-row"><strong>{{ row.currency_id }}</strong><span>净 {{ signed(row.net_change) }}</span><span>已解释 {{ signed(row.explained_change) }}</span><span :class="{ warning: row.unattributed_change !== 0 }">未归因 {{ signed(row.unattributed_change) }}</span></div></div>
    <p v-else class="empty">此区间还没有可计算的两次有效快照。</p>
  </section>
</template>

<style scoped>
.history-page { display:grid; gap:14px; padding:24px; background:#1b1b18; border:1px solid #3d3a32; border-radius:16px; }
header,.actions,.history-row { display:flex; align-items:center; justify-content:space-between; gap:12px; } header p { margin:0 0 4px; color:#c99749; font-size:11px; font-weight:800; letter-spacing:.12em; } h2 { margin:0; color:#f4efdf; font:700 24px Georgia,serif; } input { min-height:38px; padding:0 8px; color:#ebe2d0; background:#131410; border:1px solid #60584a; border-radius:7px; } .actions { justify-content:start; } button { min-height:38px; padding:0 12px; color:#1b150d; font-weight:800; background:#d79d46; border:0; border-radius:7px; cursor:pointer; } button.secondary { color:#e1d7c5; background:#282720; border:1px solid #595446; }.history-rows { border-top:1px solid #37352e; }.history-row { padding:12px 0; color:#c9c2b2; border-bottom:1px solid #37352e; font-size:13px; }.history-row strong { color:#f4efdf; text-transform:capitalize; }.warning { color:#e7a452; }.empty { margin:0; color:#aaa394; font-size:13px; } button:focus-visible,input:focus-visible { outline:2px solid #f2ce8a; outline-offset:2px; } @media (max-width:620px) { header { align-items:start; flex-direction:column; }.history-row { display:grid; grid-template-columns:1fr 1fr; } }
</style>
