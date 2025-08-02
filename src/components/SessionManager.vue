<script setup lang="ts">
import { ref, computed } from 'vue'
import { useSessionsStore, type Session } from '../stores/sessions'
import SessionCard from './SessionCard.vue'
import CreateSessionModal from './CreateSessionModal.vue'
import Terminal from './Terminal.vue'

const sessionsStore = useSessionsStore()

const showCreateModal = ref(false)
const showEditModal = ref(false)
const editingSession = ref<Session | null>(null)
const searchQuery = ref('')

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

function openCreateModal() {
  console.log('SessionManager: Opening create modal')
  showCreateModal.value = true
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

function backToSessions() {
  sessionsStore.closeSession()
}</script>

<template>
  <div class="session-manager">
    <!-- Terminal View -->
    <div v-if="sessionsStore.activeSession" class="terminal-view">
      <div class="terminal-header">
        <button @click="backToSessions" class="btn btn-secondary">
          ← Back to Sessions
        </button>
        <h3>{{ sessionsStore.activeSession.name }}</h3>
      </div>
      <Terminal 
        :session-id="sessionsStore.activeSession.id" 
        :session-name="sessionsStore.activeSession.name"
        :protocol="sessionsStore.activeSession.protocol"
      />
    </div>

    <!-- Sessions List View -->
    <div v-else class="sessions-view">
      <div class="manager-header">
        <div class="header-left">
          <h2 class="manager-title">SSH Sessions</h2>
          <div class="session-count">
            {{ sessionsStore.sessions.length }} session(s)
          </div>
        </div>
        
        <div class="header-right">
          <div class="search-box">
            <input
              v-model="searchQuery"
              type="text"
              placeholder="Search sessions..."
              class="search-input"
            />
            <span class="search-icon">🔍</span>
          </div>
          
          <button class="btn btn-primary" @click="openCreateModal">
            <span>➕</span>
            New Session
          </button>
        </div>
      </div>

      <div class="sessions-grid" v-if="filteredSessions.length > 0">
        <SessionCard
          v-for="session in filteredSessions"
          :key="session.id"
          :session="session"
          @edit-session="openEditModal"
        />
      </div>

      <div class="empty-state" v-else-if="sessionsStore.sessions.length === 0">
        <div class="empty-icon">📡</div>
        <h3 class="empty-title">No SSH Sessions</h3>
        <p class="empty-description">
          Create your first SSH session to get started connecting to remote servers.
        </p>
        <button class="btn btn-primary" @click="openCreateModal">
          <span>➕</span>
          Create First Session
        </button>
      </div>

      <div class="no-results" v-else>
        <div class="empty-icon">🔍</div>
        <h3 class="empty-title">No Results Found</h3>
        <p class="empty-description">
          No sessions match your search query "{{ searchQuery }}".
        </p>
      </div>

      <div v-if="sessionsStore.error" class="error-message">
        <span class="error-icon">⚠️</span>
        {{ sessionsStore.error }}
      </div>
    </div>

    <!-- Create Session Modal -->
    <CreateSessionModal
      v-if="showCreateModal"
      @close="closeCreateModal"
    />

    <!-- Edit Session Modal -->
    <CreateSessionModal
      v-if="showEditModal && editingSession"
      :editing-session="editingSession"
      @close="closeEditModal"
    />
  </div>
</template>

<style scoped>
.session-manager {
  height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  overflow: hidden;
}

/* Terminal View Styles */
.terminal-view {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.terminal-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1rem 1.5rem;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.terminal-header h3 {
  color: white;
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
}

/* Sessions View Styles */
.sessions-view {
  height: 100vh;
  padding: 1.5rem;
  overflow-y: auto;
}

.manager-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 2rem;
  gap: 1rem;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.manager-title {
  color: white;
  font-size: 2rem;
  font-weight: 700;
  margin: 0;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.session-count {
  background: rgba(255, 255, 255, 0.2);
  color: white;
  padding: 0.5rem 1rem;
  border-radius: 25px;
  font-size: 0.875rem;
  font-weight: 500;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.header-right {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.search-box {
  position: relative;
  display: flex;
  align-items: center;
}

.search-input {
  padding: 0.75rem 1rem;
  padding-right: 2.5rem;
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 25px;
  background: rgba(255, 255, 255, 0.1);
  color: white;
  font-size: 0.875rem;
  width: 250px;
  backdrop-filter: blur(10px);
  transition: all 0.3s ease;
}

.search-input::placeholder {
  color: rgba(255, 255, 255, 0.7);
}

.search-input:focus {
  outline: none;
  border-color: rgba(255, 255, 255, 0.4);
  background: rgba(255, 255, 255, 0.15);
  box-shadow: 0 0 0 3px rgba(255, 255, 255, 0.1);
}

.search-icon {
  position: absolute;
  right: 1rem;
  color: rgba(255, 255, 255, 0.6);
  pointer-events: none;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1.5rem;
  border: none;
  border-radius: 25px;
  font-size: 0.875rem;
  font-weight: 600;
  text-decoration: none;
  cursor: pointer;
  transition: all 0.3s ease;
  backdrop-filter: blur(10px);
}

.btn-primary {
  background: rgba(255, 255, 255, 0.2);
  color: white;
  border: 1px solid rgba(255, 255, 255, 0.3);
}

.btn-primary:hover {
  background: rgba(255, 255, 255, 0.3);
  border-color: rgba(255, 255, 255, 0.4);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.1);
  color: white;
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.2);
  border-color: rgba(255, 255, 255, 0.3);
}

.sessions-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: 1.5rem;
  animation: fadeInUp 0.6s ease-out;
}

.empty-state,
.no-results {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 4rem 2rem;
  min-height: 400px;
  animation: fadeIn 0.6s ease-out;
}

.empty-icon {
  font-size: 4rem;
  margin-bottom: 1.5rem;
  opacity: 0.8;
}

.empty-title {
  color: white;
  font-size: 1.5rem;
  font-weight: 600;
  margin: 0 0 0.75rem 0;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.empty-description {
  color: rgba(255, 255, 255, 0.8);
  font-size: 1rem;
  line-height: 1.6;
  margin: 0 0 2rem 0;
  max-width: 400px;
}

.error-message {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  background: rgba(220, 53, 69, 0.9);
  color: white;
  padding: 1rem 1.5rem;
  border-radius: 12px;
  margin-top: 1rem;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(220, 53, 69, 0.3);
}

.error-icon {
  font-size: 1.25rem;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (max-width: 768px) {
  .manager-header {
    flex-direction: column;
    align-items: stretch;
    gap: 1.5rem;
  }

  .header-right {
    flex-direction: column;
    gap: 1rem;
  }

  .search-input {
    width: 100%;
  }

  .sessions-grid {
    grid-template-columns: 1fr;
  }
}
</style>