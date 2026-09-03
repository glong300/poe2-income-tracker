<script setup lang="ts">
import { computed, ref } from "vue";

type Candidate = {
  id: number;
  candidate: {
    realm_hint: "international" | "china" | null;
    confidence: number;
    entries: { currency_id: string; quantity: number }[];
  };
};

const props = defineProps<{ candidate: Candidate }>();
const emit = defineEmits<{
  confirm: [id: number, entries: { currencyId: string; quantity: number }[]];
  reject: [id: number];
}>();
const quantities = ref<Record<string, string>>({});

const rows = computed(() =>
  props.candidate.candidate.entries.map((entry) => ({
    ...entry,
    quantity: quantities.value[entry.currency_id] ?? String(entry.quantity),
  })),
);

function confirm() {
  const entries = rows.value.map((entry) => ({
    currencyId: entry.currency_id,
    quantity: Number(entry.quantity),
  }));
  if (entries.some((entry) => !Number.isInteger(entry.quantity) || entry.quantity < 0)) return;
  emit("confirm", props.candidate.id, entries);
}
</script>

<template>
  <article class="candidate-review">
    <header>
      <div>
        <p>LOCAL CANDIDATE</p>
        <h3>待确认快照</h3>
      </div>
      <span>{{ candidate.candidate.confidence }}% 置信度</span>
    </header>
    <p class="hint">建议区服：{{ candidate.candidate.realm_hint === "china" ? "国服" : candidate.candidate.realm_hint === "international" ? "国际服" : "未识别" }}。确认前可修正数量。</p>
    <label v-for="row in rows" :key="row.currency_id">
      <span>{{ row.currency_id }}</span>
      <input
        :name="`candidate-${candidate.id}-${row.currency_id}`"
        :value="row.quantity"
        type="number"
        min="0"
        step="1"
        @input="quantities[row.currency_id] = ($event.target as HTMLInputElement).value"
      />
    </label>
    <footer>
      <button class="reject" type="button" @click="emit('reject', candidate.id)">丢弃</button>
      <button data-testid="confirm-candidate" type="button" @click="confirm">确认入账</button>
    </footer>
  </article>
</template>

<style scoped>
.candidate-review { display:grid; gap:12px; padding:18px; background:#201c16; border:1px solid #5a4930; border-radius:14px; }
header, footer, label { display:flex; align-items:center; justify-content:space-between; gap:12px; }
header p { margin:0; color:#d39c49; font-size:10px; font-weight:800; letter-spacing:.14em; }
h3 { margin:4px 0 0; color:#f4efdf; font:700 19px Georgia,serif; }
header > span { padding:5px 8px; color:#e7c888; background:#342714; border-radius:999px; font-size:11px; font-weight:700; }
.hint { margin:0; color:#b5aa96; font-size:13px; line-height:1.5; }
label { padding-top:10px; color:#eee5d4; border-top:1px solid #4a4031; font-size:13px; text-transform:capitalize; }
input { width:94px; padding:7px 8px; color:#f4efdf; background:#151512; border:1px solid #655b47; border-radius:6px; }
button { min-height:36px; padding:0 12px; color:#1a150d; font-weight:800; background:#d69b42; border:0; border-radius:7px; cursor:pointer; }
.reject { color:#d9c8b0; background:transparent; border:1px solid #675a46; }
button:focus-visible, input:focus-visible { outline:2px solid #f2ce8a; outline-offset:2px; }
</style>
