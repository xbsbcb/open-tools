<template>
  <div class="app-overlay">
    <SearchBox />
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import SearchBox from "./components/SearchBox.vue";

async function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    await getCurrentWindow().hide();
  }
}

onMounted(() => window.addEventListener("keydown", handleKeydown));
onUnmounted(() => window.removeEventListener("keydown", handleKeydown));
</script>

<style>
html,
body {
  margin: 0;
  padding: 0;
  background: transparent;
  overflow: hidden;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}
</style>

<style scoped>
.app-overlay {
  width: 100vw;
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  padding-top: 120px;
  box-sizing: border-box;
}
</style>
