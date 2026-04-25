import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export interface SearchResult {
  name: string
  icon: string
  path: string
  score: number
}

export const useAppStore = defineStore('app', () => {
  const query = ref('')
  const results = ref<SearchResult[]>([])
  const loading = ref(false)
  const calcResult = ref<string | null>(null)
  const sidecarReady = ref(false)
  const sidecarResponses = ref<Record<string, unknown>>({})

  // Listen for async responses from the Deno sidecar
  listen<string>('sidecar-output', (event) => {
    try {
      const msg = JSON.parse(event.payload)
      if (msg?.id) {
        sidecarResponses.value = {
          ...sidecarResponses.value,
          [msg.id]: msg,
        }
      }
    } catch {
      // malformed JSON — ignore
    }
  })

  listen<void>('sidecar-ready', () => {
    sidecarReady.value = true
  })

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

  async function sendToSidecar(payload: object) {
    return invoke<string>('send_to_sidecar', {
      payload: JSON.stringify(payload),
    }).catch((e) => {
      console.error('send_to_sidecar error:', e)
      return null
    })
  }

  return { query, results, loading, search, calcResult, sidecarReady, sidecarResponses, sendToSidecar }
})
