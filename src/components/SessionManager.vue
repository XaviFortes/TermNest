<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useSessionsStore, type Session } from '../stores/sessions'
import CreateSessionModal from './CreateSessionModal.vue'
import TabsContainer from './TabsContainer.vue'
import QuickActionsMenu from './QuickActionsMenu.vue'
import ContextMenu from './ContextMenu.vue'

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

function openCreateModal() {
  console.log('SessionManager: Opening create modal')
  showCreateModal.value = true
  showQuickActions.value = false
}

function closeCreateModal() {
  console.log('SessionManager: Closing create modal')
  showCreateModal.value = false
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
  sessionsStore.openSession(session)
}

function showSessionContextMenu(event: MouseEvent, session: Session) {
  event.preventDefault()
  contextMenuSession.value = session
  contextMenuPosition.value = { x: event.clientX, y: event.clientY }
  showContextMenu.value = true
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
    <!-- Sidebar for Sessions -->
    <div class="sidebar" :class="{ 'collapsed': sidebarCollapsed }">
      <div class="sidebar-header">
        <div class="sidebar-title" v-if="!sidebarCollapsed">
          <h2>Sessions</h2>
          <div class="session-count">
            {{ sessionsStore.sessions.length }}
          </div>
        </div>
        <button class="sidebar-toggle" @click="toggleSidebar" :title="sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'">
          <span v-if="sidebarCollapsed">▶</span>
          <span v-else>◀</span>
        </button>
      </div>

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
          
          <button class="btn btn-primary btn-full" @click="openCreateModal">
            <span>➕</span>
            New Session
          </button>
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
              <div class="session-name">{{ session.name }}</div>
              <div class="session-details">
                {{ session.username }}@{{ session.host }}
                <span class="status-debug" v-if="connectionStatuses[session.id] !== 'disconnected'">
                  • {{ connectionStatuses[session.id] }}
                </span>
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
      <div class="main-header">
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
      </div>
      <TabsContainer @create-session="openCreateModal" />
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
.session-manager {
  height: 100%;
  display: flex;
  background: var(--bg-primary);
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

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem;
  border-bottom: 1px solid var(--border-color);
  min-height: 60px;
}

.sidebar-title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.sidebar-title h2 {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--text-primary);
}

.session-count {
  background: var(--text-accent);
  color: white;
  padding: 0.25rem 0.5rem;
  border-radius: 12px;
  font-size: 0.75rem;
  font-weight: 500;
  min-width: 20px;
  text-align: center;
}

.sidebar-toggle {
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0.5rem;
  border-radius: 6px;
  transition: all 0.2s ease;
  font-size: 0.875rem;
}

.sidebar-toggle:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
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
  margin-bottom: 1rem;
}

.search-input {
  width: 100%;
  padding: 0.75rem 1rem;
  padding-right: 2.5rem;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.875rem;
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
  right: 1rem;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-secondary);
  pointer-events: none;
}

.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  border: none;
  border-radius: 8px;
  font-size: 0.875rem;
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
  padding: 0.5rem 0;
}

.session-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  cursor: pointer;
  transition: all 0.2s ease;
  border-left: 3px solid transparent;
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
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-secondary);
  transition: all 0.2s ease;
}

.session-status .status-dot.connected {
  background: #28a745 !important;
  box-shadow: 0 0 6px rgba(40, 167, 69, 0.5);
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
  margin-bottom: 0.25rem;
}

.session-details {
  font-size: 0.75rem;
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
  padding: 0.25rem 0.5rem;
  background: var(--bg-primary);
  border-radius: 4px;
  font-size: 0.75rem;
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