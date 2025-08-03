<template>
  <div class="status-bar">
    <div class="status-left">
      <div class="status-item">
        <span class="status-icon">📊</span>
        <span>{{ activeSessionsCount }} Active</span>
      </div>
      <div class="status-item">
        <span class="status-icon">🔗</span>
        <span>{{ connectedSessionsCount }} Connected</span>
      </div>
      <div class="status-item" v-if="currentSession">
        <span class="status-icon">💻</span>
        <span>{{ currentSession.name }}</span>
      </div>
    </div>
    
    <div class="status-right">
      <div class="status-item" v-if="isConnecting">
        <div class="connecting-spinner"></div>
        <span>Connecting...</span>
      </div>
      <div class="status-item">
        <span class="status-icon">⚡</span>
        <span>{{ memoryUsage }}</span>
      </div>
      <div class="status-item version-item" @click="checkForUpdates" :title="versionTooltip">
        <span class="status-icon">🏷️</span>
        <span>v{{ currentVersion }}</span>
        <span v-if="hasNewVersion" class="update-indicator">●</span>
      </div>
      <div class="status-item">
        <span class="status-icon">🕒</span>
        <span>{{ currentTime }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useSessionsStore } from '../stores/sessions'
import { getVersion } from '@tauri-apps/api/app'

const sessionsStore = useSessionsStore()
const currentTime = ref('')
const memoryUsage = ref('--')

// Version tracking
const currentVersion = ref('0.1.1-rc.1') // Fallback version
const latestVersion = ref('')
const hasNewVersion = ref(false)
const isCheckingVersion = ref(false)
const lastVersionCheck = ref<Date | null>(null)

const versionTooltip = computed(() => {
  if (isCheckingVersion.value) return 'Checking for updates...'
  if (hasNewVersion.value) return `New version ${latestVersion.value} available! Click to check GitHub.`
  if (lastVersionCheck.value) {
    return `Version ${currentVersion.value} - Last checked: ${lastVersionCheck.value.toLocaleTimeString()}`
  }
  return `Version ${currentVersion.value} - Click to check for updates`
})

const activeSessionsCount = computed(() => sessionsStore.activeSessions.length)
const connectedSessionsCount = computed(() => sessionsStore.connectedSessions.length)
const currentSession = computed(() => sessionsStore.currentActiveSession)

const isConnecting = computed(() => {
  return sessionsStore.activeSessions.some(session => {
    const status = sessionsStore.getConnectionStatus(session.id)
    return status?.status === 'connecting'
  })
})

function updateTime() {
  const now = new Date()
  currentTime.value = now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

function updateMemoryUsage() {
  if (performance && (performance as any).memory) {
    const memory = (performance as any).memory
    const used = Math.round(memory.usedJSHeapSize / 1024 / 1024)
    memoryUsage.value = `${used}MB`
  } else {
    memoryUsage.value = 'N/A'
  }
}

async function checkForUpdates() {
  if (isCheckingVersion.value) return
  
  isCheckingVersion.value = true
  try {
    const response = await fetch('https://api.github.com/repos/XaviFortes/TermNest/releases/latest')
    if (!response.ok) {
      console.warn('Failed to check for updates:', response.statusText)
      return
    }
    
    const release = await response.json()
    const githubVersion = release.tag_name // This includes the 'v' prefix like 'v0.1.2'
    
    latestVersion.value = githubVersion
    lastVersionCheck.value = new Date()
    localStorage.setItem('lastVersionCheck', lastVersionCheck.value.toISOString())
    
    // Compare versions (remove 'v' prefix for comparison)
    const currentVersionClean = currentVersion.value
    const latestVersionClean = githubVersion.startsWith('v') ? githubVersion.substring(1) : githubVersion
    
    // Simple version comparison (works for semantic versioning)
    hasNewVersion.value = isNewerVersion(latestVersionClean, currentVersionClean)
    
    if (hasNewVersion.value) {
      console.log(`New version available: ${githubVersion}`)
      // Open GitHub releases page
      window.open(`https://github.com/XaviFortes/TermNest/releases/latest`, '_blank')
    } else {
      console.log('You have the latest version!')
    }
  } catch (error) {
    console.error('Error checking for updates:', error)
  } finally {
    isCheckingVersion.value = false
  }
}

async function checkForUpdatesQuietly() {
  if (isCheckingVersion.value) return
  
  isCheckingVersion.value = true
  try {
    const response = await fetch('https://api.github.com/repos/XaviFortes/TermNest/releases/latest')
    if (!response.ok) {
      console.warn('Failed to check for updates:', response.statusText)
      return
    }
    
    const release = await response.json()
    const githubVersion = release.tag_name
    
    latestVersion.value = githubVersion
    lastVersionCheck.value = new Date()
    localStorage.setItem('lastVersionCheck', lastVersionCheck.value.toISOString())
    
    // Compare versions (remove 'v' prefix for comparison)
    const currentVersionClean = currentVersion.value
    const latestVersionClean = githubVersion.startsWith('v') ? githubVersion.substring(1) : githubVersion
    
    // Simple version comparison (works for semantic versioning)
    hasNewVersion.value = isNewerVersion(latestVersionClean, currentVersionClean)
    
    if (hasNewVersion.value) {
      console.log(`New version available: ${githubVersion} (current: ${currentVersion.value})`)
    }
  } catch (error) {
    console.warn('Background version check failed:', error)
  } finally {
    isCheckingVersion.value = false
  }
}

function isNewerVersion(latest: string, current: string): boolean {
  const latestParts = latest.split('.').map(Number)
  const currentParts = current.split('.').map(Number)
  
  for (let i = 0; i < Math.max(latestParts.length, currentParts.length); i++) {
    const latestPart = latestParts[i] || 0
    const currentPart = currentParts[i] || 0
    
    if (latestPart > currentPart) return true
    if (latestPart < currentPart) return false
  }
  
  return false
}

let timeInterval: number
let memoryInterval: number

onMounted(async () => {
  updateTime()
  updateMemoryUsage()
  
  // Get the app version from Tauri
  try {
    const appVersion = await getVersion()
    currentVersion.value = appVersion
  } catch (error) {
    console.warn('Failed to get app version:', error)
    // Keep the fallback version
  }
  
  timeInterval = setInterval(updateTime, 1000)
  memoryInterval = setInterval(updateMemoryUsage, 5000)
  
  // Check for updates on startup (but not too frequently)
  const lastCheck = localStorage.getItem('lastVersionCheck')
  const lastCheckTime = lastCheck ? new Date(lastCheck) : null
  const now = new Date()
  
  // Only check if it's been more than 24 hours since last check
  if (!lastCheckTime || (now.getTime() - lastCheckTime.getTime()) > 24 * 60 * 60 * 1000) {
    setTimeout(() => {
      checkForUpdatesQuietly()
    }, 2000) // Wait 2 seconds after app startup
  }
})

onUnmounted(() => {
  if (timeInterval) clearInterval(timeInterval)
  if (memoryInterval) clearInterval(memoryInterval)
})
</script>

<style scoped>
.status-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 1rem;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
  font-size: 0.75rem;
  color: var(--text-secondary);
  min-height: 32px;
}

.status-left,
.status-right {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-weight: 500;
}

.status-icon {
  font-size: 0.875rem;
}

.connecting-spinner {
  width: 12px;
  height: 12px;
  border: 2px solid var(--border-color);
  border-top: 2px solid var(--text-accent);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.version-item {
  position: relative;
  cursor: pointer;
  transition: all 0.2s ease;
  border-radius: 4px;
  padding: 0.25rem 0.5rem;
  margin: -0.25rem -0.5rem;
}

.version-item:hover {
  background: var(--bg-secondary);
  color: var(--text-accent);
}

.update-indicator {
  color: #ff6b35;
  font-size: 0.625rem;
  animation: pulse 2s infinite;
  margin-left: 0.25rem;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

@media (max-width: 768px) {
  .status-bar {
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
  }
  
  .status-left,
  .status-right {
    gap: 0.75rem;
  }
}
</style>
