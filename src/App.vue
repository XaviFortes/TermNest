<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useSessionsStore } from './stores/sessions'
import { useSettingsStore } from './stores/settings'
import SessionManager from './components/SessionManager.vue'
import SettingsModal from './components/SettingsModal.vue'

const sessionsStore = useSessionsStore()
const settingsStore = useSettingsStore()
const showSettingsModal = ref(false)

onMounted(async () => {
  // Initialize stores
  await settingsStore.initializeStore()
  await sessionsStore.loadSessions()
  
  // Set up event listeners
  sessionsStore.initializeEventListeners()
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
</style>
