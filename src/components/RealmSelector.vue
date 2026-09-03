<script setup lang="ts">
import { ref } from "vue";

type Realm = "international" | "china";

const props = defineProps<{ modelValue: Realm }>();
const emit = defineEmits<{ change: [realm: Realm] }>();
const selected = ref<Realm>(props.modelValue);

function choose(realm: Realm) {
  selected.value = realm;
  emit("change", realm);
}
</script>

<template>
  <div class="realm-selector" aria-label="经济区服">
    <span data-testid="realm-badge" class="realm-badge">{{ selected === "china" ? "国服" : "国际服" }}</span>
    <div class="realm-options" role="group" aria-label="切换区服">
      <button type="button" value="international" :aria-pressed="selected === 'international'" @click="choose('international')">国际服</button>
      <button type="button" value="china" :aria-pressed="selected === 'china'" @click="choose('china')">国服</button>
    </div>
  </div>
</template>

<style scoped>
.realm-selector { display: flex; align-items: center; gap: 10px; }
.realm-badge { padding: 5px 9px; color: #f5d9a2; background: #3b2d16; border: 1px solid #8d6429; border-radius: 999px; font-size: 12px; font-weight: 700; }
.realm-options { display: flex; padding: 3px; background: #1d1d1a; border: 1px solid #403d35; border-radius: 10px; }
button { min-height: 32px; padding: 0 10px; color: #bdb6a7; background: transparent; border: 0; border-radius: 7px; cursor: pointer; }
button[aria-pressed="true"] { color: #18140e; font-weight: 700; background: #d79d46; }
button:focus-visible { outline: 2px solid #f5d9a2; outline-offset: 2px; }
</style>
