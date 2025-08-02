<script setup lang="ts">
import { computed } from 'vue'
import { useSessionsStore, type Session } from '../stores/sessions'

interface Props {
  session: Session
}

interface Emits {
  (e: 'edit-session', session: Session): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()
const sessionsStore = useSessionsStore()

const connectionStatus = computed(() => {
  return sessionsStore.getConnectionStatus(props.session.id)
})

const isConnected = computed(() => {
  return connectionStatus.value?.status === 'connected'
})

const isConnecting = computed(() => {
  return connectionStatus.value?.status === 'connecting'
})

const protocolIcon = computed(() => {
  if (props.session.protocol === 'SSH') return '🔗'
  if (props.session.protocol === 'SFTP') return '📁'
  if (props.session.protocol === 'RDP') return '🖥️'
  if (props.session.protocol === 'VNC') return '👁️'
  if (props.session.protocol === 'Telnet') return '📟'
  return '🔗'
})

const protocolName = computed(() => {
  return props.session.protocol || 'SSH'
})

const authMethodDisplay = computed(() => {
  if (props.session.auth_method === 'Password') return 'Password'
  if (typeof props.session.auth_method === 'object' && 'PublicKey' in props.session.auth_method) return 'SSH Key'
  if (props.session.auth_method === 'Agent') return 'SSH Agent'
  return 'Unknown'
})

async function connect() {
  try {
    await sessionsStore.connectToSession(props.session.id)
    // Open the session in terminal view
    sessionsStore.openSession(props.session)
  } catch (error) {
    console.error('Failed to connect:', error)
  }
}

async function disconnect() {
  console.log('SessionCard: Disconnecting session:', props.session.id)
  console.log('SessionCard: Current connection status before disconnect:', connectionStatus.value)
  try {
    await sessionsStore.disconnectSession(props.session.id)
    console.log('SessionCard: Disconnect completed')
    
    // Check status after a small delay to see if reactive update happened
    setTimeout(() => {
      console.log('SessionCard: Connection status after disconnect (delayed check):', connectionStatus.value)
      console.log('SessionCard: isConnected computed:', isConnected.value)
      console.log('SessionCard: isConnecting computed:', isConnecting.value)
    }, 100)
  } catch (error) {
    console.error('SessionCard: Failed to disconnect:', error)
  }
}

function editSession() {
  emit('edit-session', props.session)
}

async function deleteSession() {
  if (confirm(`Are you sure you want to delete the session "${props.session.name}"?`)) {
    try {
      await sessionsStore.deleteSession(props.session.id)
    } catch (error) {
      console.error('Failed to delete session:', error)
    }
  }
}

function formatDate(dateString: string) {
  return new Date(dateString).toLocaleDateString()
}
</script>

<template>
  <div class="session-card" :class="{ connected: isConnected, connecting: isConnecting }">
    <div class="card-header">
      <div class="session-info">
        <div class="session-name">
          <span class="protocol-icon">{{ protocolIcon }}</span>
          {{ session.name }}
        </div>
        <div class="session-details">
          {{ session.username }}@{{ session.host }}:{{ session.port }}
        </div>
      </div>
      
      <div class="session-status">
        <div class="status-indicator" :class="connectionStatus?.status || 'disconnected'">
          <span class="status-dot"></span>
          {{ connectionStatus?.status || 'disconnected' }}
        </div>
      </div>
    </div>

    <div class="card-body">
      <div class="session-meta">
        <div class="meta-item">
          <span class="meta-label">Protocol:</span>
          <span class="meta-value">{{ protocolName }}</span>
        </div>
        <div class="meta-item">
          <span class="meta-label">Auth:</span>
          <span class="meta-value">{{ authMethodDisplay }}</span>
        </div>
        <div class="meta-item">
          <span class="meta-label">Created:</span>
          <span class="meta-value">{{ formatDate(session.created_at) }}</span>
        </div>
        <div class="meta-item" v-if="session.last_used">
          <span class="meta-label">Last used:</span>
          <span class="meta-value">{{ formatDate(session.last_used) }}</span>
        </div>
      </div>

      <div v-if="connectionStatus?.message" class="status-message">
        {{ connectionStatus.message }}
      </div>
    </div>

    <div class="card-actions">
      <button 
        v-if="!isConnected && !isConnecting"
        @click="connect"
        class="btn btn-primary"
        :disabled="sessionsStore.isLoading"
      >
        <span>▶️</span>
        Connect
      </button>
      
      <button 
        v-else-if="isConnected"
        @click="disconnect"
        class="btn btn-secondary"
      >
        <span>⏹️</span>
        Disconnect
      </button>
      
      <button 
        v-else
        class="btn btn-secondary"
        disabled
      >
        <span class="spinner">⏳</span>
        Connecting...
      </button>

      <div class="action-menu">
        <button class="btn btn-secondary btn-sm" title="Settings">
          <span>⚙️</span>
        </button>
        <button class="btn btn-secondary btn-sm" @click="editSession" title="Edit Session">
          <span>📝</span>
        </button>
        <button class="btn btn-secondary btn-sm" @click="deleteSession" title="Delete Session">
          <span>🗑️</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.session-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
  padding: 1.5rem;
  transition: all 0.2s ease;
  position: relative;
  overflow: hidden;
}

.session-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-hover);
  border-color: var(--text-accent);
}

.session-card.connected {
  border-color: #28a745;
  background: linear-gradient(135deg, var(--bg-secondary) 0%, rgba(40, 167, 69, 0.05) 100%);
}

.session-card.connecting {
  border-color: #ffc107;
  background: linear-gradient(135deg, var(--bg-secondary) 0%, rgba(255, 193, 7, 0.05) 100%);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 1rem;
  gap: 1rem;
}

.session-info {
  flex: 1;
  min-width: 0;
}

.session-name {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 0.25rem;
}

.protocol-icon {
  font-size: 1.25rem;
  flex-shrink: 0;
}

.session-details {
  font-size: 0.875rem;
  color: var(--text-secondary);
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
}

.session-status {
  flex-shrink: 0;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.75rem;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #6c757d;
}

.status-indicator.connected .status-dot {
  background: #28a745;
  animation: pulse 2s infinite;
}

.status-indicator.connecting .status-dot {
  background: #ffc107;
  animation: pulse 1s infinite;
}

.status-indicator.disconnected .status-dot {
  background: #6c757d;
}

@keyframes pulse {
  0% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
  100% {
    opacity: 1;
  }
}

.card-body {
  margin-bottom: 1.5rem;
}

.session-meta {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.meta-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.875rem;
}

.meta-label {
  color: var(--text-secondary);
  font-weight: 500;
}

.meta-value {
  color: var(--text-primary);
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
}

.status-message {
  background: var(--bg-tertiary);
  padding: 0.75rem;
  border-radius: 0.375rem;
  font-size: 0.875rem;
  color: var(--text-secondary);
  border-left: 3px solid var(--text-accent);
}

.card-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1rem;
}

.action-menu {
  display: flex;
  gap: 0.5rem;
}

.btn-sm {
  padding: 0.375rem 0.75rem;
  font-size: 0.75rem;
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 480px) {
  .session-card {
    padding: 1rem;
  }
  
  .card-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.5rem;
  }
  
  .session-meta {
    grid-template-columns: 1fr;
  }
  
  .card-actions {
    flex-direction: column;
    align-items: stretch;
    gap: 1rem;
  }
  
  .action-menu {
    justify-content: center;
  }
}
</style>
