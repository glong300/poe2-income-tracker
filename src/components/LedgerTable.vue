<script setup lang="ts">
defineProps<{
  rows: { currencyId: string; netChange: number; explainedChange: number; unattributedChange: number }[];
}>();

function signed(value: number) {
  return value > 0 ? `+${value}` : String(value);
}
</script>

<template>
  <section class="ledger-table" aria-labelledby="ledger-title">
    <div class="section-heading">
      <div>
        <p>DAILY BREAKDOWN</p>
        <h2 id="ledger-title">今日通货变动</h2>
      </div>
      <span>{{ rows.length }} 项</span>
    </div>
    <div v-if="rows.length" class="table" role="table">
      <div v-for="row in rows" :key="row.currencyId" class="table-row" role="row">
        <strong role="cell">{{ row.currencyId }}</strong>
        <span role="cell">净变化 {{ signed(row.netChange) }}</span>
        <span role="cell">已解释 {{ signed(row.explainedChange) }}</span>
        <span :class="{ warning: row.unattributedChange !== 0 }" role="cell">未归因 {{ signed(row.unattributedChange) }}</span>
      </div>
    </div>
    <p v-else class="empty-state">保存两次快照后，这里会显示当日变化。</p>
  </section>
</template>

<style scoped>
.ledger-table { padding: 24px; background: #1b1b18; border: 1px solid #3d3a32; border-radius: 16px; }
.section-heading, .table-row { display: flex; justify-content: space-between; gap: 16px; align-items: center; }
.section-heading { margin-bottom: 18px; }
.section-heading p { margin: 0 0 4px; color: #c99749; letter-spacing: .12em; font-size: 11px; font-weight: 700; }
h2 { margin: 0; color: #f4efdf; font: 700 22px Georgia, serif; }
.section-heading > span { color: #a39d8e; font-size: 13px; }
.table { border-top: 1px solid #34322c; }
.table-row { padding: 14px 0; color: #c9c2b2; border-bottom: 1px solid #34322c; font-size: 13px; }
.table-row strong { color: #f4efdf; text-transform: capitalize; }
.warning { color: #e7a452; }
.empty-state { margin: 0; color: #aaa394; font-size: 14px; }
@media (max-width: 620px) { .table-row { display: grid; grid-template-columns: 1fr 1fr; } }
</style>
