<template>
  <div class="plugin-manager">
    <div class="plugin-tabs">
      <button :class="{ active: tab === 'installed' }" @click="tab = 'installed'">已安装</button>
      <button :class="{ active: tab === 'discover' }" @click="tab = 'discover'">发现</button>
    </div>

    <div v-if="tab === 'installed'" class="plugin-panel">
      <a-button type="primary" size="small" :loading="refreshing" @click="refreshInstalled">
        刷新
      </a-button>
      <div v-if="installed.length === 0" class="empty">暂无已安装插件</div>
      <div v-for="p in installed" :key="p.id" class="plugin-card">
        <div class="plugin-card__info">
          <span class="plugin-card__name">{{ p.name }}</span>
          <span class="plugin-card__version">v{{ p.version }}</span>
          <span class="plugin-card__desc">{{ p.description }}</span>
        </div>
        <div class="plugin-card__actions">
          <a-button size="small" @click="load(p.id)">加载</a-button>
          <a-button size="small" danger @click="uninstall(p.id)">卸载</a-button>
        </div>
      </div>
    </div>

    <div v-if="tab === 'discover'" class="plugin-panel">
      <a-input-search
        v-model:value="discoveryQuery"
        placeholder="搜索插件..."
        @search="discover"
      />
      <div v-if="discovered.length === 0 && !searching" class="empty">
        点击搜索按钮发现插件
      </div>
      <div v-for="p in discovered" :key="p.repo" class="plugin-card">
        <div class="plugin-card__info">
          <span class="plugin-card__name">{{ p.name }}</span>
          <span class="plugin-card__author">by {{ p.author }} ⭐ {{ p.stars }}</span>
          <span class="plugin-card__desc">{{ p.description }}</span>
        </div>
        <div class="plugin-card__actions">
          <a-button size="small" type="primary" :loading="installing === p.repo" @click="install(p.repo)">
            安装
          </a-button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface PluginMeta {
  name: string
  repo: string
  description: string
  author: string
  stars: number
}

interface InstalledPlugin {
  id: string
  name: string
  version: string
  description: string
  enabled: boolean
}

const tab = ref<'installed' | 'discover'>('installed')

// Installed panel
const installed = ref<InstalledPlugin[]>([])
const refreshing = ref(false)

async function refreshInstalled() {
  refreshing.value = true
  try {
    installed.value = await invoke<InstalledPlugin[]>('list_plugins')
  } catch (e) {
    console.error('list_plugins error:', e)
  } finally {
    refreshing.value = false
  }
}

async function load(id: string) {
  try {
    await invoke('load_plugin', { pluginId: id })
  } catch (e) {
    console.error('load_plugin error:', e)
  }
}

async function uninstall(id: string) {
  try {
    await invoke('uninstall_plugin', { id })
    installed.value = installed.value.filter((p) => p.id !== id)
  } catch (e) {
    console.error('uninstall_plugin error:', e)
  }
}

// Discover panel
const discoveryQuery = ref('')
const discovered = ref<PluginMeta[]>([])
const searching = ref(false)
const installing = ref<string | null>(null)

async function discover() {
  searching.value = true
  try {
    discovered.value = await invoke<PluginMeta[]>('discover_plugins')
  } catch (e) {
    console.error('discover_plugins error:', e)
  } finally {
    searching.value = false
  }
}

async function install(repoUrl: string) {
  installing.value = repoUrl
  try {
    await invoke<InstalledPlugin>('install_plugin', { repoUrl })
    await refreshInstalled()
  } catch (e) {
    console.error('install_plugin error:', e)
  } finally {
    installing.value = null
  }
}
</script>

<style scoped>
.plugin-manager {
  width: 640px;
  border-radius: 12px;
  overflow: hidden;
  backdrop-filter: blur(20px);
  background: rgba(30, 30, 30, 0.85);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  color: rgba(255, 255, 255, 0.9);
}

.plugin-tabs {
  display: flex;
  gap: 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.plugin-tabs button {
  flex: 1;
  padding: 12px;
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.6);
  font-size: 14px;
  cursor: pointer;
  transition: color 0.15s;
}

.plugin-tabs button.active {
  color: #61dafb;
  border-bottom: 2px solid #61dafb;
}

.plugin-panel {
  padding: 12px;
  max-height: 400px;
  overflow-y: auto;
}

.empty {
  padding: 24px;
  text-align: center;
  color: rgba(255, 255, 255, 0.4);
  font-size: 13px;
}

.plugin-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  margin: 4px 0;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.06);
  transition: background 0.1s;
}

.plugin-card:hover {
  background: rgba(255, 255, 255, 0.1);
}

.plugin-card__info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow: hidden;
}

.plugin-card__name {
  font-size: 14px;
  font-weight: 500;
}

.plugin-card__version,
.plugin-card__author {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.45);
}

.plugin-card__desc {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.55);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.plugin-card__actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}
</style>
