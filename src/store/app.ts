import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface SearchResult {
  name: string
  icon: string   // base64 or path
  path: string
  score: number
}

export const useAppStore = defineStore('app', () => {
  const query = ref('')
  const results = ref<SearchResult[]>([])
  const loading = ref(false)

  async function search(q: string) {
    loading.value = true
    try {
      results.value = await invoke<SearchResult[]>('search_apps', { query: q })
    } catch (e) {
      console.error('search_apps error:', e)
      results.value = []
    } finally {
      loading.value = false
    }
  }

  return { query, results, loading, search }
})
