<template>
  <div class="quick-actions-menu" v-if="show" @click.self="close">
    <div class="menu-content">
      <div class="menu-header">
        <h3>Quick Actions</h3>
        <button class="close-btn" @click="close">×</button>
      </div>
      
      <div class="menu-section">
        <h4>Sessions</h4>
        <div class="action-item" @click="$emit('new-session')">
          <div class="action-icon">➕</div>
          <div class="action-details">
            <div class="action-title">New Session</div>
            <div class="action-shortcut">Ctrl+N</div>
          </div>
        </div>
        
        <div class="action-item" @click="$emit('close-current')" v-if="hasActiveSessions">
          <div class="action-icon">✕</div>
          <div class="action-details">
            <div class="action-title">Close Current Tab</div>
            <div class="action-shortcut">Ctrl+W</div>
          </div>
        </div>
        
        <div class="action-item" @click="$emit('close-all')" v-if="hasActiveSessions">
          <div class="action-icon">⏹</div>
          <div class="action-details">
            <div class="action-title">Close All Tabs</div>
            <div class="action-shortcut">Ctrl+Shift+W</div>
          </div>
        </div>
      </div>
      
      <div class="menu-section" v-if="recentSessions.length > 0">
        <h4>Recent Sessions</h4>
        <div 
          v-for="session in recentSessions.slice(0, 5)" 
          :key="session.id"
          class="action-item session-item"
          @click="$emit('open-session', session)"
        >
          <div class="action-icon">🖥️</div>
          <div class="action-details">
            <div class="action-title">{{ session.name }}</div>
            <div class="action-subtitle">{{ session.username }}@{{ session.host }}</div>
          </div>
        </div>
      </div>
      
      <div class="menu-section">
        <h4>Navigation</h4>
        <div class="action-item" @click="$emit('next-tab')" v-if="hasActiveSessions">
          <div class="action-icon">→</div>
          <div class="action-details">
            <div class="action-title">Next Tab</div>
            <div class="action-shortcut">Ctrl+Tab</div>
          </div>
        </div>
        
        <div class="action-item" @click="$emit('prev-tab')" v-if="hasActiveSessions">
          <div class="action-icon">←</div>
          <div class="action-details">
            <div class="action-title">Previous Tab</div>
            <div class="action-shortcut">Ctrl+Shift+Tab</div>
          </div>
        </div>
        
        <div class="action-item" @click="$emit('toggle-sidebar')">
          <div class="action-icon">📋</div>
          <div class="action-details">
            <div class="action-title">Toggle Sidebar</div>
            <div class="action-shortcut">Ctrl+B</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Session } from '../stores/sessions'

defineProps<{
  show: boolean
  hasActiveSessions: boolean
  recentSessions: Session[]
}>()

defineEmits<{
  close: []
  'new-session': []
  'close-current': []
  'close-all': []
  'open-session': [session: Session]
  'next-tab': []
  'prev-tab': []
  'toggle-sidebar': []
}>()

function close() {
  // Will be handled by parent
}
</script>

<style scoped>
.quick-actions-menu {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  z-index: 2000;
  backdrop-filter: blur(4px);
  padding-top: 10vh;
}

.menu-content {
  background: var(--bg-primary);
  border-radius: 12px;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
  max-width: 400px;
  width: 90%;
  max-height: 80vh;
  overflow-y: auto;
  border: 1px solid var(--border-color);
}

.menu-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.5rem 1.5rem 1rem 1.5rem;
  border-bottom: 1px solid var(--border-color);
}

.menu-header h3 {
  margin: 0;
  color: var(--text-primary);
  font-size: 1.25rem;
  font-weight: 600;
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
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.menu-section {
  padding: 1rem 0;
  border-bottom: 1px solid var(--border-color);
}

.menu-section:last-child {
  border-bottom: none;
}

.menu-section h4 {
  margin: 0 0 0.75rem 0;
  padding: 0 1.5rem;
  color: var(--text-secondary);
  font-size: 0.875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.action-item {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.75rem 1.5rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.action-item:hover {
  background: var(--bg-secondary);
}

.action-icon {
  font-size: 1.25rem;
  width: 24px;
  text-align: center;
  flex-shrink: 0;
}

.action-details {
  flex: 1;
  min-width: 0;
}

.action-title {
  font-weight: 500;
  color: var(--text-primary);
  margin-bottom: 0.25rem;
}

.action-subtitle {
  font-size: 0.875rem;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.action-shortcut {
  font-size: 0.75rem;
  color: var(--text-secondary);
  background: var(--bg-tertiary);
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  border: 1px solid var(--border-color);
  font-weight: 500;
  margin-left: auto;
  white-space: nowrap;
}

.session-item .action-details {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
}

.session-item .action-title {
  margin-bottom: 0.25rem;
}
</style>
