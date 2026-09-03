<script setup lang="ts">
import { ref } from "vue";

defineProps<{ currencies: { id: string; name: string }[] }>();
const emit = defineEmits<{ submit: [input: { currencyId: string; quantity: number; direction: "inflow" | "outflow"; kind: "trade" | "exchange" | "crafting" | "other" }] }>();
const currencyId = ref("exalted");
const quantity = ref("");
const direction = ref<"inflow" | "outflow">("inflow");
const kind = ref<"trade" | "exchange" | "crafting" | "other">("trade");
const error = ref("");

function submit() {
  const parsed = Number(quantity.value);
  if (!Number.isInteger(parsed) || parsed <= 0) { error.value = "数量必须为正整数"; return; }
  error.value = "";
  emit("submit", { currencyId: currencyId.value, quantity: parsed, direction: direction.value, kind: kind.value });
}
</script>

<template>
  <form class="adjustment-form" @submit.prevent="submit">
    <header><p>LEDGER ADJUSTMENT</p><h2>记录收支调整</h2></header>
    <label>通货<select v-model="currencyId" name="adjustment-currency"><option v-for="currency in currencies" :key="currency.id" :value="currency.id">{{ currency.name }}</option></select></label>
    <label>数量<input v-model="quantity" name="adjustment-quantity" type="number" min="1" step="1" /></label>
    <div class="two-columns"><label>方向<select v-model="direction" name="adjustment-direction"><option value="inflow">收入</option><option value="outflow">支出</option></select></label><label>类型<select v-model="kind" name="adjustment-kind"><option value="trade">交易</option><option value="exchange">兑换</option><option value="crafting">制作</option><option value="other">其他</option></select></label></div>
    <p v-if="error" class="error" role="alert">{{ error }}</p><button type="submit">记入账本</button>
  </form>
</template>

<style scoped>
.adjustment-form { display:grid; gap:11px; padding:20px; background:#161714; border:1px solid #3d3c33; border-radius:14px; } header p { margin:0; color:#c78637; font-size:10px; letter-spacing:.14em; font-weight:800; } h2 { margin:4px 0 0; color:#eee5d4; font:700 20px Georgia,serif; } label { display:grid; gap:6px; color:#cfc6b4; font-size:12px; } input,select { min-height:35px; width:100%; padding:0 8px; color:#eee5d4; background:#24231f; border:1px solid #5f5948; border-radius:6px; } .two-columns { display:grid; grid-template-columns:1fr 1fr; gap:8px; }.error { margin:0; color:#ffab85; font-size:12px; } button { min-height:39px; color:#1c1710; font-weight:800; background:#d89b42; border:0; border-radius:7px; cursor:pointer; } button:focus-visible,input:focus-visible,select:focus-visible { outline:2px solid #f2ce8a; outline-offset:2px; }
</style>
