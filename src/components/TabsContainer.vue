<template>
  <div class="tabs-container">
    <!-- Tab Bar -->
    <div class="tab-bar" v-if="activeSessions.length > 0">
      <div class="tabs-wrapper">
        <div
          v-for="session in activeSessions"
          :key="session.id"
          class="tab"
          :class="{ 
            'active': session.id === currentActiveSessionId,
            'connected': tabConnectionStatuses[session.id] === 'connected',
            'connecting': tabConnectionStatuses[session.id] === 'connecting',
            'error': tabConnectionStatuses[session.id] === 'error'
          }"
          @click="switchToSession(session.id)"
        >
          <div class="tab-content">
            <div class="tab-status">
              <div class="status-indicator"></div>
            </div>
            <div class="tab-info">
              <div class="tab-title">{{ session.name }}</div>
              <div class="tab-subtitle">{{ session.username }}@{{ session.host }}</div>
            </div>
            <button 
              class="tab-close" 
              @click.stop="closeSessionTab(session.id)"
              :title="`Close ${session.name}`"
            >
              ×
            </button>
          </div>
        </div>
      </div>
      
      <!-- Add Session Button -->
      <button class="add-session-btn" @click="showSessionSelector" title="Open new session">
        <span class="add-icon">+</span>
      </button>
    </div>

    <!-- Terminal Content Area -->
    <div class="terminal-content-area" :class="{ 'with-tabs': activeSessions.length > 0 }">
      <div
        v-for="session in activeSessions"
        :key="session.id"
        class="terminal-pane"
        :class="{ 'active': session.id === currentActiveSessionId }"
      >
        <Terminal
          :session-id="session.id"
          :session-name="session.name"
          :protocol="session.protocol"
        />
      </div>
      
      <!-- Empty state when no sessions -->
      <div v-if="activeSessions.length === 0" class="empty-terminal-state">
        <div class="empty-content">
          <div class="empty-icon">💻</div>
          <h3>No Active Sessions</h3>
          <p>Select a session from the sidebar to start a new terminal session</p>
        </div>
      </div>
    </div>

    <!-- Session Selector Modal -->
    <div v-if="showSelector" class="session-selector-modal" @click.self="hideSessionSelector">
      <div class="session-selector">
        <div class="selector-header">
          <h3>Select Session to Open</h3>
          <button class="close-btn" @click="hideSessionSelector">×</button>
        </div>
        <div class="selector-content">
          <div class="session-list">
            <div
              v-for="session in availableSessions"
              :key="session.id"
              class="session-option"
              @click="openSessionFromSelector(session)"
            >
              <div class="session-icon">🖥️</div>
              <div class="session-details">
                <div class="session-name">{{ session.name }}</div>
                <div class="session-info">{{ session.username }}@{{ session.host }}:{{ session.port }}</div>
              </div>
              <div class="session-protocol">{{ session.protocol }}</div>
            </div>
          </div>
          <div v-if="availableSessions.length === 0" class="no-sessions">
            <p>No available sessions. Create a new session to get started.</p>
            <button class="btn btn-primary" @click="$emit('create-session')">
              Create New Session
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useSessionsStore } from '../stores/sessions'
import Terminal from './Terminal.vue'

const emit = defineEmits(['create-session'])

const sessionsStore = useSessionsStore()
const showSelector = ref(false)

const activeSessions = computed(() => sessionsStore.activeSessions)
const currentActiveSessionId = computed(() => sessionsStore.currentActiveSessionId)

const availableSessions = computed(() => {
  return sessionsStore.sessions.filter(session => 
    !activeSessions.value.find(active => active.id === session.id)
  )
})

// Force reactivity for connection statuses in tabs
const tabConnectionStatuses = computed(() => {
  const connections = sessionsStore.activeConnections
  const statuses: Record<string, string> = {}
  
  activeSessions.value.forEach(session => {
    const status = connections[session.id]
    statuses[session.id] = status?.status || 'disconnected'
  })
  
  console.log('TabsContainer: computed tabConnectionStatuses:', statuses)
  return statuses
})

function switchToSession(sessionId: string) {
  sessionsStore.switchToSession(sessionId)
}

function closeSessionTab(sessionId: string) {
  // Disconnect the session first
  const connectionStatus = sessionsStore.getConnectionStatus(sessionId)
  if (connectionStatus?.status === 'connected') {
    sessionsStore.disconnectSession(sessionId)
  }
  // Close the tab
  sessionsStore.closeSession(sessionId)
}

function showSessionSelector() {
  showSelector.value = true
}

function hideSessionSelector() {
  showSelector.value = false
}

function openSessionFromSelector(session: any) {
  sessionsStore.openSession(session)
  hideSessionSelector()
}
</script>

<style scoped>
.tabs-container {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
}

/* Tab Bar Styles */
.tab-bar {
  display: flex;
  align-items: center;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  padding: 0;
  min-height: 48px;
  flex-shrink: 0;
  overflow: hidden;
}

.tabs-wrapper {
  display: flex;
  flex: 1;
  overflow-x: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.tabs-wrapper::-webkit-scrollbar {
  display: none;
}

.tab {
  display: flex;
  align-items: center;
  min-width: 200px;
  max-width: 250px;
  height: 48px;
  background: var(--bg-tertiary);
  border-right: 1px solid var(--border-color);
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
  overflow: hidden;
}

.tab.active {
  background: var(--bg-primary);
  border-bottom: 2px solid var(--text-accent);
}

.tab:hover:not(.active) {
  background: var(--bg-primary);
  transform: translateY(-1px);
}

.tab-content {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
  width: 100%;
  min-width: 0;
}

.tab-status {
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.status-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-secondary);
  transition: background-color 0.2s ease;
}

.tab.connected .status-indicator {
  background: #28a745;
  box-shadow: 0 0 6px rgba(40, 167, 69, 0.5);
}

.tab.connecting .status-indicator {
  background: #ffc107;
  animation: pulse 1.5s infinite;
}

.tab.error .status-indicator {
  background: #dc3545;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.tab-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
}

.tab-title {
  font-weight: 500;
  font-size: 0.875rem;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  width: 100%;
}

.tab-subtitle {
  font-size: 0.75rem;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  width: 100%;
}

.tab-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: 4px;
  font-size: 16px;
  font-weight: bold;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.tab-close:hover {
  background: rgba(220, 53, 69, 0.1);
  color: #dc3545;
}

.add-session-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 48px;
  height: 48px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
  border-left: 1px solid var(--border-color);
}

.add-session-btn:hover {
  background: var(--bg-primary);
  color: var(--text-accent);
}

.add-icon {
  font-size: 18px;
  font-weight: bold;
}

/* Terminal Content Area */
.terminal-content-area {
  flex: 1;
  position: relative;
  overflow: hidden;
}

.terminal-content-area.with-tabs {
  height: calc(100% - 48px);
}

.terminal-pane {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: none;
}

.terminal-pane.active {
  display: block;
}

.empty-terminal-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  background: var(--bg-primary);
}

.empty-content {
  text-align: center;
  color: var(--text-secondary);
}

.empty-icon {
  font-size: 4rem;
  margin-bottom: 1rem;
}

.empty-content h3 {
  margin: 0 0 0.5rem 0;
  color: var(--text-primary);
}

.empty-content p {
  margin: 0;
  font-size: 0.9rem;
}

/* Session Selector Modal */
.session-selector-modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(4px);
}

.session-selector {
  background: var(--bg-primary);
  border-radius: 12px;
  box-shadow: var(--shadow-hover);
  max-width: 500px;
  width: 90%;
  max-height: 60vh;
  overflow: hidden;
  border: 1px solid var(--border-color);
}

.selector-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 1.5rem;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.selector-header h3 {
  margin: 0;
  color: var(--text-primary);
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  color: var(--text-secondary);
  cursor: pointer;
  width: 32px;
  height: 32px;
  border-radius: 6px;
  transition: all 0.2s ease;
}

.close-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.selector-content {
  max-height: calc(60vh - 80px);
  overflow-y: auto;
}

.session-list {
  padding: 0.5rem 0;
}

.session-option {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.75rem 1.5rem;
  cursor: pointer;
  transition: background-color 0.2s ease;
}

.session-option:hover {
  background: var(--bg-secondary);
}

.session-icon {
  font-size: 1.5rem;
  flex-shrink: 0;
}

.session-details {
  flex: 1;
  min-width: 0;
}

.session-name {
  font-weight: 500;
  color: var(--text-primary);
  margin-bottom: 0.25rem;
}

.session-info {
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.session-protocol {
  padding: 0.25rem 0.5rem;
  background: var(--bg-tertiary);
  border-radius: 4px;
  font-size: 0.75rem;
  color: var(--text-secondary);
  font-weight: 500;
}

.no-sessions {
  padding: 2rem 1.5rem;
  text-align: center;
  color: var(--text-secondary);
}

.no-sessions p {
  margin-bottom: 1rem;
}

.btn {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 6px;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-primary {
  background: var(--text-accent);
  color: white;
}

.btn-primary:hover {
  transform: translateY(-1px);
  box-shadow: var(--shadow-hover);
}
</style>
