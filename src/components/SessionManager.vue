<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useSessionsStore, type Session } from '../stores/sessions'
import { invoke } from '@tauri-apps/api/core'
import CreateSessionModal from './CreateSessionModal.vue'
import TabsContainer from './TabsContainer.vue'
import QuickActionsMenu from './QuickActionsMenu.vue'
import ContextMenu from './ContextMenu.vue'

// Props
const emit = defineEmits<{
  openSettings: []
}>()

const sessionsStore = useSessionsStore()

const showCreateModal = ref(false)
const showEditModal = ref(false)
const editingSession = ref<Session | null>(null)
const searchQuery = ref('')
const sidebarCollapsed = ref(false)
const showQuickActions = ref(false)
const showContextMenu = ref(false)
const contextMenuPosition = ref({ x: 0, y: 0 })
const contextMenuSession = ref<Session | null>(null)

const filteredSessions = computed(() => {
  if (!searchQuery.value) {
    return sessionsStore.sessions
  }
  
  const query = searchQuery.value.toLowerCase()
  return sessionsStore.sessions.filter(session => 
    session.name.toLowerCase().includes(query) ||
    session.host.toLowerCase().includes(query) ||
    session.username.toLowerCase().includes(query)
  )
})

const recentSessions = computed(() => {
  return sessionsStore.sessions
    .filter(session => session.last_used)
    .sort((a, b) => new Date(b.last_used!).getTime() - new Date(a.last_used!).getTime())
    .slice(0, 5)
})

// Force reactivity for connection statuses
const connectionStatuses = computed(() => {
  // Explicitly watch the activeConnections object
  const connections = sessionsStore.activeConnections
  const statuses: Record<string, string> = {}
  
  sessionsStore.sessions.forEach(session => {
    const status = connections[session.id]
    statuses[session.id] = status?.status || 'disconnected'
  })
  
  console.log('SessionManager: computed connectionStatuses:', statuses)
  return statuses
})

// Count active connections per session (including multiple connections)
const connectionCounts = computed(() => {
  const counts: Record<string, number> = {}
  
  sessionsStore.sessions.forEach(session => {
    // Count how many active sessions are based on this session (original + additional connections)
    const baseId = session.id
    const relatedSessions = sessionsStore.activeSessions.filter(activeSession => 
      activeSession.id === baseId || activeSession.id.startsWith(`${baseId}_conn_`)
    )
    counts[session.id] = relatedSessions.length
  })
  
  return counts
})

function openCreateModal() {
  console.log('SessionManager: Opening create modal')
  showCreateModal.value = true
  showQuickActions.value = false
}

function closeCreateModal() {
  console.log('SessionManager: Closing create modal')
  showCreateModal.value = false
}

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

function openSettings() {
  emit('openSettings')
}

function openEditModal(session: Session) {
  console.log('SessionManager: Opening edit modal for session:', session)
  editingSession.value = session
  showEditModal.value = true
}

function closeEditModal() {
  console.log('SessionManager: Closing edit modal')
  showEditModal.value = false
  editingSession.value = null
}

function toggleSidebar() {
  sidebarCollapsed.value = !sidebarCollapsed.value
}

function openSession(session: Session) {
  // Check if the session is already active
  const existingSession = sessionsStore.activeSessions.find(s => s.id === session.id)
  
  if (existingSession) {
    // If already active, create a new connection instance
    const timestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
    const newConnectionSession = {
      ...session,
      id: `${session.id}_conn_${Date.now()}`,
      name: `${session.name} (${timestamp})`,
    }
    console.log('Creating additional connection for session:', session.name)
    sessionsStore.openSession(newConnectionSession)
  } else {
    // If not active, open normally
    console.log('Opening new session:', session.name)
    sessionsStore.openSession(session)
  }
}

function showSessionContextMenu(event: MouseEvent, session: Session) {
  event.preventDefault()
  contextMenuSession.value = session
  
  // Calculate position to prevent menu from going off-screen
  // More accurate context menu dimensions (based on actual ContextMenu component)
  const menuWidth = 180 // More accurate width
  const menuHeight = 350 // Height including all menu items and separators
  const viewportWidth = window.innerWidth
  const viewportHeight = window.innerHeight
  
  let x = event.clientX
  let y = event.clientY
  
  // Adjust x position if menu would go off right edge
  if (x + menuWidth > viewportWidth) {
    x = Math.max(5, viewportWidth - menuWidth - 10)
  }
  
  // Adjust y position if menu would go off bottom edge
  if (y + menuHeight > viewportHeight) {
    y = Math.max(5, viewportHeight - menuHeight - 10)
  }
  
  // Ensure menu doesn't go off top or left edges
  x = Math.max(5, x)
  y = Math.max(5, y)
  
  contextMenuPosition.value = { x, y }
  showContextMenu.value = true
  
  // Add click outside to close
  setTimeout(() => {
    document.addEventListener('click', hideContextMenu, { once: true })
  })
}

function hideContextMenu() {
  showContextMenu.value = false
  contextMenuSession.value = null
}

function connectToSession() {
  if (contextMenuSession.value) {
    openSession(contextMenuSession.value)
    hideContextMenu()
  }
}

function editSession() {
  if (contextMenuSession.value) {
    openEditModal(contextMenuSession.value)
    hideContextMenu()
  }
}

function duplicateSession() {
  if (contextMenuSession.value) {
    const session = contextMenuSession.value
    const duplicatedSession = {
      ...session,
      name: `${session.name} (Copy)`,
      id: '', // Will be generated in createSession
      created_at: ''
    }
    delete (duplicatedSession as any).id
    delete (duplicatedSession as any).created_at
    sessionsStore.createSession(duplicatedSession)
    hideContextMenu()
  }
}

function createNewConnection() {
  if (contextMenuSession.value) {
    const session = contextMenuSession.value
    // Create a temporary session instance with a unique ID for this connection
    const newConnectionSession = {
      ...session,
      id: `${session.id}_conn_${Date.now()}`, // Unique ID for this connection instance
      name: `${session.name} (${new Date().toLocaleTimeString()})`, // Add timestamp to differentiate
    }
    
    // Open this new connection instance directly
    sessionsStore.openSession(newConnectionSession)
    hideContextMenu()
  }
}

function exportSession() {
  if (contextMenuSession.value) {
    const session = contextMenuSession.value
    const exportData = {
      name: session.name,
      host: session.host,
      port: session.port,
      username: session.username,
      protocol: session.protocol,
      auth_method: session.auth_method
    }
    
    const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${session.name.replace(/[^a-zA-Z0-9]/g, '_')}_session.json`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
    hideContextMenu()
  }
}

function deleteSession() {
  if (contextMenuSession.value) {
    if (confirm(`Are you sure you want to delete the session "${contextMenuSession.value.name}"?`)) {
      sessionsStore.deleteSession(contextMenuSession.value.id)
      hideContextMenu()
    }
  }
}

// Debug function to test connection statuses
function testConnectionStatus(session: Session, status: 'connected' | 'connecting' | 'error' | 'disconnected') {
  sessionsStore.updateConnectionStatus({
    session_id: session.id,
    status: status,
    message: `Test ${status} status`
  })
}

function closeCurrentTab() {
  if (sessionsStore.currentActiveSessionId) {
    sessionsStore.closeSession(sessionsStore.currentActiveSessionId)
  }
}

function closeAllTabs() {
  sessionsStore.closeSession()
}

function nextTab() {
  const current = sessionsStore.activeSessions.findIndex(s => s.id === sessionsStore.currentActiveSessionId)
  if (current !== -1 && sessionsStore.activeSessions.length > 1) {
    const next = (current + 1) % sessionsStore.activeSessions.length
    sessionsStore.switchToSession(sessionsStore.activeSessions[next].id)
  }
}

function prevTab() {
  const current = sessionsStore.activeSessions.findIndex(s => s.id === sessionsStore.currentActiveSessionId)
  if (current !== -1 && sessionsStore.activeSessions.length > 1) {
    const prev = current === 0 ? sessionsStore.activeSessions.length - 1 : current - 1
    sessionsStore.switchToSession(sessionsStore.activeSessions[prev].id)
  }
}

function handleKeydown(event: KeyboardEvent) {
  // Quick actions menu
  if (event.ctrlKey && event.shiftKey && event.key === 'P') {
    event.preventDefault()
    showQuickActions.value = !showQuickActions.value
  }
  
  // New session
  if (event.ctrlKey && event.key === 'n') {
    event.preventDefault()
    openCreateModal()
  }
  
  // Close current tab
  if (event.ctrlKey && event.key === 'w' && !event.shiftKey) {
    event.preventDefault()
    closeCurrentTab()
  }
  
  // Close all tabs
  if (event.ctrlKey && event.shiftKey && event.key === 'W') {
    event.preventDefault()
    closeAllTabs()
  }
  
  // Toggle sidebar
  if (event.ctrlKey && event.key === 'b') {
    event.preventDefault()
    toggleSidebar()
  }
  
  // Next tab
  if (event.ctrlKey && event.key === 'Tab' && !event.shiftKey) {
    event.preventDefault()
    nextTab()
  }
  
  // Previous tab
  if (event.ctrlKey && event.shiftKey && event.key === 'Tab') {
    event.preventDefault()
    prevTab()
  }
  
  // Escape to close menus
  if (event.key === 'Escape') {
    showQuickActions.value = false
    hideContextMenu()
  }
}

function handleGlobalClick() {
  hideContextMenu()
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
  window.addEventListener('click', handleGlobalClick)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
  window.removeEventListener('click', handleGlobalClick)
})
</script>

<template>
  <div class="session-manager" @click="handleGlobalClick">
    <!-- Unified Header -->
    <div class="unified-header">
      <div class="header-left">
        <h1 class="app-title">
          <span class="title-icon">🏠</span>
          TermNest
        </h1>
        <div class="session-info">
          <button class="sidebar-toggle" @click="toggleSidebar" :title="sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'">
            <span v-if="sidebarCollapsed">▶</span>
            <span v-else>◀</span>
          </button>
          <button class="btn btn-primary btn-sm" @click="openCreateModal">
            <span>➕</span>
            <span v-if="!sidebarCollapsed">New</span>
          </button>
          <div class="session-count" v-if="!sidebarCollapsed">
            {{ sessionsStore.sessions.length }} Session{{ sessionsStore.sessions.length !== 1 ? 's' : '' }}
          </div>
        </div>
      </div>
      <div class="header-right">
        <button class="btn btn-secondary btn-sm" @click="testTauriConnection">
          Test
        </button>
        <button class="btn btn-primary btn-sm" @click="openSettings">
          Settings
        </button>
        <button class="btn btn-sm keyboard-hint"
          @click="showQuickActions = !showQuickActions"
          :title="'Quick Actions (Ctrl+Shift+P)'"
        >
          <span>⚡ Ctrl+Shift+P</span>
        </button>
      </div>
    </div>

    <!-- Main Body Container -->
    <div class="session-manager-body">
      <!-- Sidebar for Sessions -->
      <div class="sidebar" :class="{ 'collapsed': sidebarCollapsed }">
      <div class="sidebar-content" v-if="!sidebarCollapsed">
        <div class="sidebar-actions">
          <div class="search-box">
            <input
              v-model="searchQuery"
              type="text"
              placeholder="Search sessions..."
              class="search-input"
            />
            <span class="search-icon">🔍</span>
          </div>
        </div>

        <div class="sessions-list" v-if="filteredSessions.length > 0">
          <div
            v-for="session in filteredSessions"
            :key="session.id"
            class="session-item"
            :class="{
              'active': sessionsStore.activeSessions.find(s => s.id === session.id),
              'connected': connectionStatuses[session.id] === 'connected'
            }"
            @click="openSession(session)"
            @contextmenu="showSessionContextMenu($event, session)"
          >
            <div class="session-status">
              <div class="status-dot" :class="{
                'connected': connectionStatuses[session.id] === 'connected',
                'connecting': connectionStatuses[session.id] === 'connecting',
                'error': connectionStatuses[session.id] === 'error'
              }"></div>
            </div>
            <div class="session-info">
              <div class="session-name">
                {{ session.name }}
                <span v-if="connectionCounts[session.id] > 0" class="connection-badge">
                  {{ connectionCounts[session.id] }}
                </span>
              </div>
              <div class="session-details">
                {{ session.username }}@{{ session.host }}
              </div>
            </div>
            <div class="session-protocol">{{ session.protocol }}</div>
          </div>
        </div>

        <div class="empty-sessions" v-else-if="sessionsStore.sessions.length === 0">
          <div class="empty-icon">📡</div>
          <h4>No Sessions</h4>
          <p>Create your first SSH session to get started.</p>
          <button class="btn btn-primary" @click="openCreateModal">
            Create Session
          </button>
        </div>

        <div class="no-results" v-else>
          <div class="empty-icon">🔍</div>
          <h4>No Results</h4>
          <p>No sessions match "{{ searchQuery }}".</p>
        </div>

          <div v-if="sessionsStore.error" class="error-message">
            <span class="error-icon">⚠️</span>
            {{ sessionsStore.error }}
          </div>

          <!-- Debug panel -->
          <div class="debug-panel" v-if="Object.keys(connectionStatuses).length > 0 && false"> <!-- Debugging disabled -->
            <h4>Connection Status Debug</h4>
            <div v-for="(status, sessionId) in connectionStatuses" :key="sessionId" class="debug-item">
              <span class="debug-session">{{ sessionsStore.sessions.find(s => s.id === sessionId)?.name || sessionId }}</span>
              <span class="debug-status" :class="status">{{ status }}</span>
              <div class="debug-actions">
                <button @click="sessionsStore.setSessionConnected(sessionId)" class="debug-btn connected">✓</button>
                <button @click="sessionsStore.setSessionDisconnected(sessionId)" class="debug-btn disconnected">✕</button>
              </div>
            </div>
          </div>
        </div>      <!-- Collapsed sidebar content -->
      <div class="sidebar-collapsed-content" v-else>
        <div class="collapsed-sessions">
          <div
            v-for="session in sessionsStore.sessions.slice(0, 10)"
            :key="session.id"
            class="collapsed-session-item"
            :class="{
              'active': sessionsStore.activeSessions.find(s => s.id === session.id),
              'connected': connectionStatuses[session.id] === 'connected'
            }"
            @click="openSession(session)"
            :title="session.name"
          >
            <div class="status-dot" :class="{
              'connected': connectionStatuses[session.id] === 'connected',
              'connecting': connectionStatuses[session.id] === 'connecting',
              'error': connectionStatuses[session.id] === 'error'
            }"></div>
          </div>
        </div>
        <button class="collapsed-add-btn" @click="openCreateModal" title="Create new session">
          ➕
        </button>
      </div>
    </div>

      <!-- Main Content Area -->
      <div class="main-content">
        <!-- <div class="main-header">
          <div class="main-title">
            <h2 v-if="sessionsStore.activeSessions.length === 0">Welcome to TermNest</h2>
            <h2 v-else>{{ sessionsStore.activeSessions.length }} Active Session{{ sessionsStore.activeSessions.length !== 1 ? 's' : '' }}</h2>
          </div>
          <div class="main-actions">
            <button 
              class="quick-actions-btn" 
              @click="showQuickActions = !showQuickActions"
              :title="'Quick Actions (Ctrl+Shift+P)'"
            >
              ⚡
            </button>
          </div>
        </div> -->
        <TabsContainer @create-session="openCreateModal" />
      </div>
    </div>

    <!-- Modals -->
    <CreateSessionModal
      v-if="showCreateModal"
      @close="closeCreateModal"
    />

    <CreateSessionModal
      v-if="showEditModal && editingSession"
      :editing-session="editingSession"
      @close="closeEditModal"
    />

    <!-- Quick Actions Menu -->
    <QuickActionsMenu
      v-if="showQuickActions"
      :show="showQuickActions"
      :has-active-sessions="sessionsStore.activeSessions.length > 0"
      :recent-sessions="recentSessions"
      @close="showQuickActions = false"
      @new-session="openCreateModal"
      @close-current="closeCurrentTab"
      @close-all="closeAllTabs"
      @open-session="openSession"
      @next-tab="nextTab"
      @prev-tab="prevTab"
      @toggle-sidebar="toggleSidebar"
    />

    <!-- Context Menu -->
    <ContextMenu
      v-if="showContextMenu"
      :show="showContextMenu"
      :position="contextMenuPosition"
      @connect="connectToSession"
      @new-connection="createNewConnection"
      @edit="editSession"
      @duplicate="duplicateSession"
      @export="exportSession"
      @delete="deleteSession"
      @test-connected="() => testConnectionStatus(contextMenuSession!, 'connected')"
      @test-connecting="() => testConnectionStatus(contextMenuSession!, 'connecting')"
      @test-error="() => testConnectionStatus(contextMenuSession!, 'error')"
    />
  </div>
</template>

<style scoped>
/* Unified Header Styles */
.unified-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 0.75rem;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
  min-height: 50px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.app-title {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.title-icon {
  font-size: 1.25rem;
}

.session-info {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.session-count {
  background: var(--text-accent);
  color: white;
  padding: 0.1875rem 0.375rem;
  border-radius: 8px;
  font-size: 0.6875rem;
  font-weight: 500;
}

.keyboard-hint {
  font-size: 0.6875rem;
  color: var(--text-secondary);
  background: var(--bg-primary);
  padding: 0.25rem 0.375rem;
  border-radius: 4px;
  border: 1px solid var(--border-color);
}

.keyboard-hint:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.65);
  background: var(--bg-tertiary);
  color: var(--text-accent);
}

.btn-sm {
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
  border-radius: 4px;
}

.sidebar-toggle {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  color: var(--text-primary);
  padding: 0.25rem 0.375rem;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.75rem;
  transition: all 0.2s ease;
}

.sidebar-toggle:hover {
  background: var(--bg-primary);
}

/* Session Manager Layout */
.session-manager {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
  overflow: hidden;
}

/* Layout container for sidebar and main content */
.session-manager-body {
  flex: 1;
  display: flex;
  overflow: hidden;
}

/* Sidebar Styles */
.sidebar {
  width: 300px;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  transition: width 0.3s ease;
  flex-shrink: 0;
}

.sidebar.collapsed {
  width: 60px;
}

.sidebar-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.sidebar-actions {
  padding: 1rem;
  border-bottom: 1px solid var(--border-color);
}

.search-box {
  position: relative;
  margin-bottom: 0.75rem; /* Reduced from 1rem */
}

.search-input {
  width: 100%;
  padding: 0.5rem 0.75rem; /* Reduced from 0.75rem 1rem */
  padding-right: 2rem; /* Reduced from 2.5rem */
  border: 1px solid var(--border-color);
  border-radius: 6px; /* Reduced from 8px */
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.8125rem; /* Reduced from 0.875rem */
  transition: all 0.2s ease;
}

.search-input:focus {
  outline: none;
  border-color: var(--text-accent);
  box-shadow: 0 0 0 3px rgba(70, 130, 180, 0.1);
}

.search-input::placeholder {
  color: var(--text-secondary);
}

.search-icon {
  position: absolute;
  right: 0.75rem; /* Reduced from 1rem */
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-secondary);
  pointer-events: none;
}

.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.375rem; /* Reduced from 0.5rem */
  padding: 0.5rem 0.75rem; /* Reduced from 0.75rem 1rem */
  border: none;
  border-radius: 6px; /* Reduced from 8px */
  font-size: 0.8125rem; /* Reduced from 0.875rem */
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  text-decoration: none;
}

.btn-primary {
  background: var(--text-accent);
  color: white;
}

.btn-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.btn-full {
  width: 100%;
}

.sessions-list {
  flex: 1;
  overflow-y: auto;
  padding: 0.375rem 0; /* Reduced from 0.5rem 0 */
}

.session-item {
  display: flex;
  align-items: center;
  gap: 0.5rem; /* Reduced from 0.75rem */
  padding: 0.5rem 0.75rem; /* Reduced from 0.75rem 1rem */
  cursor: pointer;
  transition: all 0.2s ease;
  border-left: 2px solid transparent; /* Reduced from 3px */
}

.session-item:hover {
  background: var(--bg-tertiary);
}

.session-item.active {
  background: var(--bg-tertiary);
  border-left-color: var(--text-accent);
}

.session-status {
  flex-shrink: 0;
}

.session-status .status-dot {
  width: 6px; /* Reduced from 8px */
  height: 6px; /* Reduced from 8px */
  border-radius: 50%;
  background: var(--text-secondary);
  transition: all 0.2s ease;
}

.session-status .status-dot.connected {
  background: #28a745 !important;
  box-shadow: 0 0 4px rgba(40, 167, 69, 0.5); /* Reduced from 6px */
}

.session-status .status-dot.connecting {
  background: #ffc107 !important;
  animation: pulse 1.5s infinite;
}

.session-status .status-dot.error {
  background: #dc3545 !important;
}

@keyframes pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.7; transform: scale(1.2); }
}

.session-info {
  flex: 1;
  min-width: 0;
}

.session-name {
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 0.1875rem; /* Reduced from 0.25rem */
  display: flex;
  align-items: center;
  gap: 0.375rem; /* Reduced from 0.5rem */
}

.connection-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px; /* Reduced from 20px */
  height: 18px; /* Reduced from 20px */
  background: var(--text-accent);
  color: white;
  font-size: 0.6875rem; /* Reduced from 0.75rem */
  font-weight: 600;
  border-radius: 8px; /* Reduced from 10px */
  padding: 0 0.1875rem; /* Reduced from 0 0.25rem */
  line-height: 1;
}

.session-details {
  font-size: 0.6875rem; /* Reduced from 0.75rem */
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.status-debug {
  color: var(--text-accent);
  font-weight: 500;
  text-transform: capitalize;
}

.session-protocol {
  padding: 0.1875rem 0.375rem; /* Reduced from 0.25rem 0.5rem */
  background: var(--bg-primary);
  border-radius: 3px; /* Reduced from 4px */
  font-size: 0.6875rem; /* Reduced from 0.75rem */
  color: var(--text-secondary);
  font-weight: 500;
  border: 1px solid var(--border-color);
}

/* Collapsed sidebar styles */
.sidebar-collapsed-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 1rem 0;
}

.collapsed-sessions {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  width: 100%;
  align-items: center;
  overflow-y: auto;
}

.collapsed-session-item {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: var(--bg-primary);
  border: 2px solid transparent;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.collapsed-session-item:hover {
  background: var(--bg-tertiary);
}

.collapsed-session-item.active {
  border-color: var(--text-accent);
}

.collapsed-session-item .status-dot {
  position: absolute;
  bottom: -2px;
  right: -2px;
  width: 10px;
  height: 10px;
  border: 2px solid var(--bg-secondary);
  background: var(--text-secondary);
}

.collapsed-session-item .status-dot.connected {
  background: #28a745 !important;
  box-shadow: 0 0 4px rgba(40, 167, 69, 0.5);
}

.collapsed-session-item .status-dot.connecting {
  background: #ffc107 !important;
  animation: pulse 1.5s infinite;
}

.collapsed-session-item .status-dot.error {
  background: #dc3545 !important;
}

.collapsed-add-btn {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 8px;
  background: var(--text-accent);
  color: white;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-top: 0.5rem;
}

.collapsed-add-btn:hover {
  transform: scale(1.1);
}

/* Empty states */
.empty-sessions,
.no-results {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 2rem 1rem;
  color: var(--text-secondary);
}

.empty-icon {
  font-size: 2.5rem;
  margin-bottom: 1rem;
  opacity: 0.7;
}

.empty-sessions h4,
.no-results h4 {
  margin: 0 0 0.5rem 0;
  color: var(--text-primary);
  font-size: 1.1rem;
}

.empty-sessions p,
.no-results p {
  margin: 0 0 1.5rem 0;
  font-size: 0.875rem;
  line-height: 1.4;
}

/* Error message */
.error-message {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  background: rgba(220, 53, 69, 0.1);
  color: #dc3545;
  padding: 0.75rem 1rem;
  margin: 1rem;
  border-radius: 8px;
  border: 1px solid rgba(220, 53, 69, 0.2);
  font-size: 0.875rem;
}

.error-icon {
  font-size: 1rem;
}

/* Debug panel */
.debug-panel {
  margin-top: 1rem;
  padding: 0.75rem;
  background: var(--bg-primary);
  border-radius: 6px;
  border: 1px solid var(--border-color);
}

.debug-panel h4 {
  margin: 0 0 0.5rem 0;
  font-size: 0.875rem;
  color: var(--text-primary);
}

.debug-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.25rem 0;
  font-size: 0.75rem;
  gap: 0.5rem;
}

.debug-session {
  color: var(--text-primary);
  font-weight: 500;
  flex: 1;
}

.debug-status {
  padding: 0.125rem 0.5rem;
  border-radius: 3px;
  font-weight: 500;
  text-transform: uppercase;
  min-width: 80px;
  text-align: center;
}

.debug-actions {
  display: flex;
  gap: 0.25rem;
}

.debug-btn {
  width: 20px;
  height: 20px;
  border: none;
  border-radius: 3px;
  cursor: pointer;
  font-size: 0.75rem;
  font-weight: bold;
  transition: all 0.2s ease;
}

.debug-btn.connected {
  background: #28a745;
  color: white;
}

.debug-btn.connected:hover {
  background: #218838;
}

.debug-btn.disconnected {
  background: #dc3545;
  color: white;
}

.debug-btn.disconnected:hover {
  background: #c82333;
}

.debug-status.connected {
  background: rgba(40, 167, 69, 0.1);
  color: #28a745;
}

.debug-status.connecting {
  background: rgba(255, 193, 7, 0.1);
  color: #ffc107;
}

.debug-status.error {
  background: rgba(220, 53, 69, 0.1);
  color: #dc3545;
}

.debug-status.disconnected {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
}

/* Main content area */
.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.main-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 1.5rem;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.main-title h2 {
  margin: 0;
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--text-primary);
}

.main-actions {
  display: flex;
  gap: 0.5rem;
}

.quick-actions-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 8px;
  background: var(--bg-primary);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 1.125rem;
  border: 1px solid var(--border-color);
}

.quick-actions-btn:hover {
  background: var(--text-accent);
  color: white;
  transform: scale(1.05);
}

/* Custom Scrollbars for Session Manager */
:deep(*) {
  scrollbar-width: thin;
  scrollbar-color: #555 #2d2d2d;
}

:deep(*::-webkit-scrollbar) {
  width: 8px;
  height: 8px;
}

:deep(*::-webkit-scrollbar-track) {
  background: #2d2d2d;
  border-radius: 4px;
}

:deep(*::-webkit-scrollbar-thumb) {
  background: #555;
  border-radius: 4px;
  border: 1px solid #2d2d2d;
}

:deep(*::-webkit-scrollbar-thumb:hover) {
  background: #666;
}

:deep(*::-webkit-scrollbar-thumb:active) {
  background: #777;
}

:deep(*::-webkit-scrollbar-corner) {
  background: #2d2d2d;
}

/* Sessions list specific scrollbar */
.sessions-list :deep(*::-webkit-scrollbar-thumb) {
  background: #404040;
}

.sessions-list :deep(*::-webkit-scrollbar-thumb:hover) {
  background: #505050;
}

/* Responsive design */
@media (max-width: 768px) {
  .sidebar {
    width: 250px;
  }
  
  .sidebar.collapsed {
    width: 50px;
  }
  
  .session-item {
    padding: 0.5rem 0.75rem;
  }
  
  .sidebar-actions {
    padding: 0.75rem;
  }
}
</style>