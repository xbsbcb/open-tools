<template>
  <div class="search-box">
    <a-input
      ref="inputRef"
      v-model:value="store.query"
      placeholder="搜索应用、文件..."
      class="search-input"
      :bordered="false"
      @input="onInput"
      @keydown="onInputKeydown"
      autofocus
    />
    <div v-if="store.calcResult !== null" class="calc-result">
      <span class="calc-equals">= </span>
      <span class="calc-value">{{ store.calcResult }}</span>
    </div>
    <ResultList v-if="store.results.length > 0" ref="resultListRef" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '../store/app'
import ResultList from './ResultList.vue'

const store = useAppStore()
const inputRef = ref<{ focus: () => void } | null>(null)
const resultListRef = ref<InstanceType<typeof ResultList> | null>(null)

let debounceTimer: ReturnType<typeof setTimeout> | null = null

function onInput() {
  if (debounceTimer !== null) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => {
    store.search(store.query)
    invoke('ping_sidecar', { message: store.query }).catch(() => {})
  }, 150)
}

function onInputKeydown(e: KeyboardEvent) {
  if (e.key === 'ArrowDown' || e.key === 'ArrowUp' || e.key === 'Enter') {
    resultListRef.value?.onKeydown(e)
  }
}

onMounted(() => {
  inputRef.value?.focus()
})
</script>

<style scoped>
.search-box {
  width: 640px;
  border-radius: 12px;
  overflow: hidden;
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  background: rgba(30, 30, 30, 0.75);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
}

.search-input {
  width: 640px;
  height: 52px;
  font-size: 16px;
  background: transparent !important;
  color: rgba(255, 255, 255, 0.9) !important;
  padding: 0 16px;
}

.search-input :deep(.ant-input) {
  background: transparent !important;
  color: rgba(255, 255, 255, 0.9) !important;
  font-size: 16px;
  height: 52px;
  line-height: 52px;
}

.search-input :deep(.ant-input::placeholder) {
  color: rgba(255, 255, 255, 0.4);
}

.calc-result {
  padding: 8px 16px;
  border-top: 1px solid rgba(255,255,255,0.08);
  display: flex;
  align-items: center;
  gap: 6px;
}
.calc-equals {
  color: rgba(255,255,255,0.4);
  font-size: 14px;
}
.calc-value {
  color: #61dafb;
  font-size: 20px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}
</style>
