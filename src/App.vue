<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useSessionsStore } from './stores/sessions'
import { useSettingsStore } from './stores/settings'
import SessionManager from './components/SessionManager.vue'
import SettingsModal from './components/SettingsModal.vue'
import StatusBar from './components/StatusBar.vue'

const sessionsStore = useSessionsStore()
const settingsStore = useSettingsStore()
const showSettingsModal = ref(false)

async function testTauriConnection() {
  try {
    console.log('Testing Tauri connection...')
    const result = await invoke('greet', { name: 'Test' })
    console.log('Tauri connection test result:', result)
    alert('Tauri connection working: ' + result)
  } catch (error) {
    console.error('Tauri connection test failed:', error)
    alert('Tauri connection failed: ' + error)
  }
}

onMounted(async () => {
  console.log('App: Mounting application...')
  try {
    // Initialize stores
    console.log('App: Initializing settings store...')
    await settingsStore.initializeStore()
    console.log('App: Settings store initialized')
    
    console.log('App: Loading sessions...')
    await sessionsStore.loadSessions()
    console.log('App: Sessions loaded')
    
    // Set up event listeners
    console.log('App: Setting up event listeners...')
    sessionsStore.initializeEventListeners()
    console.log('App: Application fully initialized')
  } catch (error) {
    console.error('App: Failed to initialize:', error)
  }
})

function openSettings() {
  showSettingsModal.value = true
}

function closeSettings() {
  showSettingsModal.value = false
}
</script>

<template>
  <div id="app" :class="`theme-${settingsStore.settings.theme}`">
    <header class="app-header">
      <div class="header-content">
        <h1 class="app-title">
          <span class="title-icon">🏠</span>
          TermNest
        </h1>
        <div class="header-actions">
          <button class="btn btn-secondary" @click="testTauriConnection">
            Test Connection
          </button>
          <button class="btn btn-primary" @click="openSettings">
            Settings
          </button>
          <div class="keyboard-hint">
            <span>⚡ Ctrl+Shift+P for Quick Actions</span>
          </div>
        </div>
      </div>
    </header>

    <main class="app-main">
      <SessionManager />
    </main>

    <StatusBar />

    <!-- Settings Modal -->
    <SettingsModal
      v-if="showSettingsModal"
      @close="closeSettings"
    />
  </div>
</template>

<style>
/* CSS Reset and Base Styles */
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body {
  height: 100%;
  font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
}

#app {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
  color: var(--text-primary);
  transition: background-color 0.2s ease, color 0.2s ease;
}

/* Theme Variables */
.theme-light {
  --bg-primary: #ffffff;
  --bg-secondary: #f8f9fa;
  --bg-tertiary: #e9ecef;
  --text-primary: #212529;
  --text-secondary: #6c757d;
  --text-accent: #0066cc;
  --border-color: #dee2e6;
  --shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  --shadow-hover: 0 4px 8px rgba(0, 0, 0, 0.15);
}

.theme-dark {
  --bg-primary: #1a1a1a;
  --bg-secondary: #2d2d2d;
  --bg-tertiary: #404040;
  --text-primary: #ffffff;
  --text-secondary: #b0b0b0;
  --text-accent: #4da6ff;
  --border-color: #404040;
  --shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
  --shadow-hover: 0 4px 8px rgba(0, 0, 0, 0.4);
}

.theme-system {
  --bg-primary: #ffffff;
  --bg-secondary: #f8f9fa;
  --bg-tertiary: #e9ecef;
  --text-primary: #212529;
  --text-secondary: #6c757d;
  --text-accent: #0066cc;
  --border-color: #dee2e6;
  --shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  --shadow-hover: 0 4px 8px rgba(0, 0, 0, 0.15);
}

@media (prefers-color-scheme: dark) {
  .theme-system {
    --bg-primary: #1a1a1a;
    --bg-secondary: #2d2d2d;
    --bg-tertiary: #404040;
    --text-primary: #ffffff;
    --text-secondary: #b0b0b0;
    --text-accent: #4da6ff;
    --border-color: #404040;
    --shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
    --shadow-hover: 0 4px 8px rgba(0, 0, 0, 0.4);
  }
}

/* Header Styles */
.app-header {
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  padding: 0.75rem 1rem;
  flex-shrink: 0;
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  max-width: 1200px;
  margin: 0 auto;
}

.app-title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--text-primary);
}

.title-icon {
  font-size: 1.5rem;
}

.header-actions {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.keyboard-hint {
  font-size: 0.75rem;
  color: var(--text-secondary);
  background: var(--bg-primary);
  padding: 0.5rem 0.75rem;
  border-radius: 6px;
  border: 1px solid var(--border-color);
  margin-left: 0.5rem;
}

.keyboard-hint span {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

/* Main Content */
.app-main {
  flex: 1;
  overflow: hidden;
  min-height: 0; /* Important for flex child to shrink */
}

/* Button Styles */
.btn {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 0.375rem;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  text-decoration: none;
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
}

.btn-primary {
  background: var(--text-accent);
  color: white;
}

.btn-primary:hover {
  transform: translateY(-1px);
  box-shadow: var(--shadow-hover);
}

.btn-secondary {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
}

.btn-secondary:hover {
  background: var(--bg-secondary);
}

/* Utility Classes */
.loading {
  opacity: 0.6;
  pointer-events: none;
}

.error {
  color: #dc3545;
}

.success {
  color: #28a745;
}

/* Global Custom Scrollbars */
* {
  scrollbar-width: thin;
  scrollbar-color: var(--border-color) var(--bg-primary);
}

*::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

*::-webkit-scrollbar-track {
  background: var(--bg-primary);
  border-radius: 4px;
}

*::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 4px;
  border: 1px solid var(--bg-primary);
}

*::-webkit-scrollbar-thumb:hover {
  background: var(--text-secondary);
}

*::-webkit-scrollbar-thumb:active {
  background: var(--text-primary);
}

*::-webkit-scrollbar-corner {
  background: var(--bg-primary);
}

/* Dark theme specific scrollbar adjustments */
.theme-dark *::-webkit-scrollbar-thumb {
  background: #555;
}

.theme-dark *::-webkit-scrollbar-thumb:hover {
  background: #666;
}

.theme-dark *::-webkit-scrollbar-thumb:active {
  background: #777;
}

.theme-dark *::-webkit-scrollbar-track {
  background: #2d2d2d;
}
</style>
