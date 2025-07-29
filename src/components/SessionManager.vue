<script setup lang="ts">
import { ref, computed } from 'vue'
import { useSessionsStore } from '../stores/sessions'
import SessionCard from './SessionCard.vue'
import CreateSessionModal from './CreateSessionModal.vue'

const sessionsStore = useSessionsStore()

const showCreateModal = ref(false)
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
  showCreateModal.value = true
}

function closeCreateModal() {
  showCreateModal.value = false
}
</script>

<template>
  <div class="session-manager">
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

    <!-- Create Session Modal -->
    <CreateSessionModal
      v-if="showCreateModal"
      @close="closeCreateModal"
    />
  </div>
</template>

<style scoped>
.session-manager {
  height: 100%;
  display: flex;
  flex-direction: column;
  max-width: 1200px;
  margin: 0 auto;
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
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--text-primary);
}

.session-count {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  padding: 0.25rem 0.75rem;
  border-radius: 1rem;
  font-size: 0.875rem;
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
  padding: 0.5rem 1rem;
  padding-right: 2.5rem;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.875rem;
  width: 250px;
  transition: border-color 0.2s ease;
}

.search-input:focus {
  outline: none;
  border-color: var(--text-accent);
}

.search-icon {
  position: absolute;
  right: 0.75rem;
  color: var(--text-secondary);
  pointer-events: none;
}

.sessions-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 1.5rem;
  flex: 1;
  overflow-y: auto;
  padding-bottom: 1rem;
}

.empty-state,
.no-results {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 4rem 2rem;
  flex: 1;
}

.empty-icon {
  font-size: 4rem;
  margin-bottom: 1rem;
  opacity: 0.5;
}

.empty-title {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 0.5rem;
}

.empty-description {
  color: var(--text-secondary);
  margin-bottom: 2rem;
  max-width: 400px;
  line-height: 1.5;
}

.error-message {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  background: #fee;
  color: #c53030;
  padding: 1rem;
  border-radius: 0.375rem;
  border: 1px solid #fed7d7;
  margin-top: 1rem;
}

.error-icon {
  flex-shrink: 0;
}

@media (max-width: 768px) {
  .manager-header {
    flex-direction: column;
    align-items: stretch;
    gap: 1rem;
  }
  
  .header-left,
  .header-right {
    justify-content: space-between;
  }
  
  .search-input {
    width: 100%;
  }
  
  .sessions-grid {
    grid-template-columns: 1fr;
  }
}
</style>
