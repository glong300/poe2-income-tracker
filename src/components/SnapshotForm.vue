<script setup lang="ts">
import { computed, ref } from "vue";

type Currency = { id: string; name: string };

const props = defineProps<{ currencies: Currency[] }>();
const emit = defineEmits<{ submit: [entries: { currencyId: string; quantity: number }[]] }>();
const quantities = ref<Record<string, string>>({});
const error = ref("");

const rows = computed(() =>
  props.currencies.map((currency) => ({
    currency,
    quantity: quantities.value[currency.id] ?? "0",
  })),
);

function submit() {
  const entries = rows.value.map(({ currency, quantity }) => ({
    currencyId: currency.id,
    quantity: Number(quantity),
  }));
  if (entries.some((entry) => !Number.isInteger(entry.quantity) || entry.quantity < 0)) {
    error.value = "数量必须是非负整数";
    return;
  }
  error.value = "";
  emit("submit", entries);
}
</script>

<template>
  <form class="snapshot-form" @submit.prevent="submit">
    <header>
      <p>手动快照</p>
      <h2>记录本次通货余额</h2>
    </header>
    <label v-for="row in rows" :key="row.currency.id" class="currency-row">
      <span>{{ row.currency.name }}</span>
      <input
        :name="`quantity-${row.currency.id}`"
        :value="row.quantity"
        inputmode="numeric"
        type="number"
        min="0"
        step="1"
        @input="quantities[row.currency.id] = ($event.target as HTMLInputElement).value"
      />
    </label>
    <p v-if="error" class="error" role="alert">{{ error }}</p>
    <button type="submit">保存快照</button>
  </form>
</template>

<style scoped>
.snapshot-form { display: grid; gap: 16px; max-width: 440px; padding: 28px; color: #f0eadc; background: #181816; border: 1px solid #484335; }
header p { margin: 0; color: #c78637; font-size: 12px; letter-spacing: .14em; text-transform: uppercase; }
h2 { margin: 4px 0 0; font-size: 22px; }
.currency-row { display: flex; justify-content: space-between; align-items: center; gap: 16px; }
input { width: 120px; padding: 9px; color: inherit; background: #24231f; border: 1px solid #5f5948; }
.error { margin: 0; color: #ff9b78; }
button { padding: 11px 14px; color: #1c1710; font-weight: 700; background: #d89b42; border: 0; cursor: pointer; }
</style>
