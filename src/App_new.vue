<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useSessionsStore } from './stores/sessions'
import { useSettingsStore } from './stores/settings'
import { useThemesStore } from './stores/themes'
import SessionManager from './components/SessionManager.vue'
import SettingsModal from './components/SettingsModal.vue'

const sessionsStore = useSessionsStore()
const settingsStore = useSettingsStore()
const themesStore = useThemesStore()

const showSettings = ref(false)

function openSettings() {
  showSettings.value = true
}

function closeSettings() {
  showSettings.value = false
}

onMounted(async () => {
  console.log('App mounting...')
  
  // Initialize stores in order
  await settingsStore.initializeStore()
  console.log('Settings store initialized')
  
  await themesStore.initializeThemes()
  console.log('Themes store initialized, current theme:', themesStore.currentThemeId)
  
  await sessionsStore.loadSessions()
  console.log('Sessions store initialized')
  
  // Sync theme from settings to themes store if they differ
  if (settingsStore.settings.theme && settingsStore.settings.theme !== themesStore.currentThemeId) {
    console.log('Syncing theme from settings:', settingsStore.settings.theme, 'to themes store')
    await themesStore.setTheme(settingsStore.settings.theme)
  }
  
  // Set up event listeners
  sessionsStore.initializeEventListeners()
  
  console.log('App initialization complete')
})
</script>

<template>
  <div id="app">
    <header class="app-header">
      <div class="header-content">
        <h1 class="app-title">
          <span class="title-icon">🏠</span>
          TermNest
        </h1>
        <div class="header-actions">
          <div class="theme-indicator">
            Theme: {{ themesStore.currentTheme.metadata.name }}
          </div>
          <button class="btn btn-secondary" @click="testTheme" style="font-size: 0.75rem;">
            Test
          </button>
          <button class="btn btn-primary" @click="openSettings">
            Settings
          </button>
        </div>
      </div>
    </header>

    <main class="app-main">
      <SessionManager />
    </main>

    <footer class="app-footer">
      <div class="footer-content">
        <span class="status-text">
          {{ sessionsStore.connectedSessions.length }} connection(s) active
        </span>
        <span class="version-text">v0.1.0</span>
      </div>
    </footer>

    <!-- Settings Modal -->
    <SettingsModal v-if="showSettings" @close="closeSettings" />
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
  background: var(--bg-primary, #ffffff);
  color: var(--text-primary, #212529);
  transition: background-color 0.2s ease, color 0.2s ease;
}

/* Default theme variables (fallbacks) */
:root {
  --bg-primary: #ffffff;
  --bg-secondary: #f8f9fa;
  --bg-tertiary: #e9ecef;
  --bg-quaternary: #dee2e6;
  --text-primary: #212529;
  --text-secondary: #6c757d;
  --text-accent: #0066cc;
  --text-muted: #adb5bd;
  --border-color: #dee2e6;
  --border-color-hover: #adb5bd;
  --shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  --shadow-hover: 0 4px 8px rgba(0, 0, 0, 0.15);
  --success: #28a745;
  --warning: #ffc107;
  --error: #dc3545;
  --info: #17a2b8;
  --button-primary-bg: #0066cc;
  --button-primary-text: #ffffff;
  --button-primary-hover: #0052a3;
  --button-secondary-bg: #e9ecef;
  --button-secondary-text: #212529;
  --button-secondary-hover: #dee2e6;
  --status-connected: #28a745;
  --status-connecting: #ffc107;
  --status-disconnected: #6c757d;
  --status-error: #dc3545;
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
  align-items: center;
  gap: 1rem;
}

.theme-indicator {
  font-size: 0.875rem;
  color: var(--text-secondary);
  padding: 0.25rem 0.5rem;
  background: var(--bg-tertiary);
  border-radius: 0.25rem;
  border: 1px solid var(--border-color);
}

/* Main Content */
.app-main {
  flex: 1;
  overflow: hidden;
  padding: 1rem;
}

/* Footer */
.app-footer {
  background: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
  padding: 0.5rem 1rem;
  flex-shrink: 0;
}

.footer-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  max-width: 1200px;
  margin: 0 auto;
  font-size: 0.875rem;
  color: var(--text-secondary);
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
  background: var(--button-primary-bg);
  color: var(--button-primary-text);
}

.btn-primary:hover {
  background: var(--button-primary-hover);
  transform: translateY(-1px);
  box-shadow: var(--shadow-hover);
}

.btn-secondary {
  background: var(--button-secondary-bg);
  color: var(--button-secondary-text);
  border: 1px solid var(--border-color);
}

.btn-secondary:hover {
  background: var(--button-secondary-hover);
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
</style>
