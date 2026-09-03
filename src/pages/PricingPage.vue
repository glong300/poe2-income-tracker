<script setup lang="ts">
import { ref } from "vue";

defineProps<{
  providerStatus: { provider: "PoeNinja" | "CNMarket"; availability: "AwaitingSync" | "Unavailable"; message: string } | null;
}>();
const emit = defineEmits<{ "import-csv": [csv: string] }>();
const csv = ref("");

function submit() {
  if (csv.value.trim()) emit("import-csv", csv.value.trim());
}
</script>

<template>
  <section class="pricing-page" aria-labelledby="pricing-title">
    <header class="page-heading">
      <div><p>PRICE SOURCES</p><h2 id="pricing-title">本地价格账本</h2></div>
      <span :class="{ offline: providerStatus?.availability === 'Unavailable' }">{{ providerStatus?.provider === "CNMarket" ? "国服自动源" : "国际服自动源" }}</span>
    </header>
    <p class="provider-message">{{ providerStatus?.message ?? "正在读取自动行情状态" }}</p>
    <div class="source-grid">
      <article><p class="label">自动行情</p><strong>{{ providerStatus?.availability === "AwaitingSync" ? "等待同步" : "尚未配置" }}</strong><small>不会覆盖本地已确认的价格。</small></article>
      <article><p class="label">手动确认价</p><strong>优先使用</strong><small>同一通货、时间与区服下，手动价优先。</small></article>
    </div>
    <form @submit.prevent="submit">
      <label for="price-csv">导入 CSV</label>
      <textarea id="price-csv" v-model="csv" spellcheck="false" placeholder="currency_id,value,quoted_in,captured_at&#10;exalted,12,chaos,2026-09-03T12:00:00+08:00" />
      <p>格式固定为 <code>currency_id,value,quoted_in,captured_at</code>。导入仅写入当前区服的本地数据库。</p>
      <button type="submit">导入确认价格</button>
    </form>
  </section>
</template>

<style scoped>
.pricing-page { display:grid; gap:14px; padding:24px; background:#1b1b18; border:1px solid #3d3a32; border-radius:16px; }
.page-heading { display:flex; justify-content:space-between; align-items:start; gap:12px; }
.page-heading p,.label { margin:0 0 4px; color:#c99749; font-size:11px; font-weight:800; letter-spacing:.12em; }
h2 { margin:0; color:#f4efdf; font:700 24px Georgia,serif; }
.page-heading span { padding:6px 9px; color:#dfc48c; background:#342714; border:1px solid #785b2d; border-radius:999px; font-size:11px; font-weight:700; }
.page-heading span.offline { color:#cbb4a1; background:#2b2520; border-color:#574b3d; }
.provider-message { margin:0; color:#b5aa96; font-size:14px; }
.source-grid { display:grid; grid-template-columns:repeat(2, minmax(0,1fr)); gap:10px; }
.source-grid article { padding:14px; background:#151512; border:1px solid #3b3931; border-radius:10px; }
.source-grid strong,.source-grid small { display:block; }.source-grid strong { color:#eee5d4; font:700 20px Georgia,serif; }.source-grid small { margin-top:7px; color:#a99f91; line-height:1.45; }
form { display:grid; gap:9px; padding-top:4px; } label { color:#eee5d4; font-weight:700; } textarea { min-height:130px; resize:vertical; padding:12px; color:#eee5d4; background:#121310; border:1px solid #60584a; border-radius:8px; font:13px/1.6 ui-monospace,SFMono-Regular,Menlo,monospace; } form p { margin:0; color:#aaa394; font-size:12px; } code { color:#e2bd7d; } button { justify-self:start; min-height:40px; padding:0 14px; color:#1b150d; font-weight:800; background:#d79d46; border:0; border-radius:7px; cursor:pointer; } button:focus-visible,textarea:focus-visible { outline:2px solid #f2ce8a; outline-offset:2px; } @media (max-width:620px) { .source-grid { grid-template-columns:1fr; } }
</style>
