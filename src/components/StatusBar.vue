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

const sessionsStore = useSessionsStore()
const currentTime = ref('')
const memoryUsage = ref('--')

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

let timeInterval: number
let memoryInterval: number

onMounted(() => {
  updateTime()
  updateMemoryUsage()
  
  timeInterval = setInterval(updateTime, 1000)
  memoryInterval = setInterval(updateMemoryUsage, 5000)
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
