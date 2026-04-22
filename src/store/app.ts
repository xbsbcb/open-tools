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
  const calcResult = ref<string | null>(null)

  async function search(q: string) {
    loading.value = true
    try {
      const [searchResults, calcValue] = await Promise.all([
        invoke<SearchResult[]>('search_apps', { query: q }),
        invoke<string | null>('eval_expr', { expr: q }).catch(() => null),
      ])
      results.value = searchResults
      calcResult.value = calcValue
    } catch (e) {
      console.error('search_apps error:', e)
      results.value = []
      calcResult.value = null
    } finally {
      loading.value = false
    }
  }

  return { query, results, loading, search, calcResult }
})
