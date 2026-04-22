<template>
  <div class="result-list" @keydown="onKeydown" tabindex="-1" ref="listRef">
    <a-list
      :data-source="visibleResults"
      :split="false"
      size="small"
    >
      <template #renderItem="{ item, index }">
        <a-list-item
          class="result-item"
          :class="{ 'result-item--active': index === selectedIndex }"
          @click="openItem(item)"
          @mouseenter="selectedIndex = index"
        >
          <div class="result-item__inner">
            <img
              v-if="item.icon"
              :src="item.icon.startsWith('data:') ? item.icon : `data:image/png;base64,${item.icon}`"
              class="result-item__icon"
              alt=""
            />
            <div v-else class="result-item__icon result-item__icon--placeholder" />
            <div class="result-item__text">
              <span class="result-item__name">{{ item.name }}</span>
              <span class="result-item__path">{{ item.path }}</span>
            </div>
          </div>
        </a-list-item>
      </template>
    </a-list>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '../store/app'
import type { SearchResult } from '../store/app'

const MAX_VISIBLE = 8

const store = useAppStore()
const listRef = ref<HTMLElement | null>(null)
const selectedIndex = ref(0)

const visibleResults = computed(() => store.results.slice(0, MAX_VISIBLE))

watch(() => store.results, () => {
  selectedIndex.value = 0
})

async function openItem(item: SearchResult) {
  try {
    await invoke('open_path', { path: item.path })
  } catch (e) {
    console.error('open_path error:', e)
  }
}

function onKeydown(e: KeyboardEvent) {
  const len = visibleResults.value.length
  if (len === 0) return
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    selectedIndex.value = (selectedIndex.value + 1) % len
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    selectedIndex.value = (selectedIndex.value - 1 + len) % len
  } else if (e.key === 'Enter') {
    e.preventDefault()
    const item = visibleResults.value[selectedIndex.value]
    if (item) openItem(item)
  }
}

onMounted(() => {
  listRef.value?.focus()
})

defineExpose({ onKeydown, selectedIndex })
</script>

<style scoped>
.result-list {
  outline: none;
  max-height: calc(8 * 56px);
  overflow-y: auto;
  border-radius: 0 0 12px 12px;
}

.result-item {
  padding: 8px 12px !important;
  cursor: pointer;
  border-radius: 8px;
  transition: background 0.1s;
}

.result-item--active {
  background: rgba(255, 255, 255, 0.15);
}

.result-item__inner {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
}

.result-item__icon {
  width: 32px;
  height: 32px;
  object-fit: contain;
  flex-shrink: 0;
  border-radius: 6px;
}

.result-item__icon--placeholder {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 6px;
}

.result-item__text {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.result-item__name {
  font-size: 14px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.95);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.result-item__path {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
