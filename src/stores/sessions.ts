import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export interface Session {
  id: string
  name: string
  host: string
  port: number
  username: string
  auth_method: AuthMethod
  protocol: Protocol
  created_at: string
  last_used?: string
}

export type AuthMethod = 
  | 'Password' 
  | { PublicKey: { key_path: string } }
  | 'Agent'

export type Protocol = 'SSH' | 'SFTP' | 'RDP' | 'VNC' | 'Telnet'

export interface ConnectionStatus {
  session_id: string
  status: string
  message?: string
}

export const useSessionsStore = defineStore('sessions', () => {
  // State
  const sessions = ref<Session[]>([])
  const activeConnections = ref<Map<string, ConnectionStatus>>(new Map())
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // Computed
  const connectedSessions = computed(() => {
    return sessions.value.filter(session => {
      const status = activeConnections.value.get(session.id)
      return status?.status === 'connected'
    })
  })

  // Actions
  async function loadSessions() {
    try {
      isLoading.value = true
      error.value = null
      const result = await invoke<Session[]>('list_sessions')
      sessions.value = result
    } catch (err) {
      error.value = err as string
      console.error('Failed to load sessions:', err)
    } finally {
      isLoading.value = false
    }
  }

  async function createSession(sessionData: Omit<Session, 'id' | 'created_at'>) {
    try {
      const newSession: Session = {
        ...sessionData,
        id: crypto.randomUUID(),
        created_at: new Date().toISOString(),
      }
      
      const result = await invoke<Session>('create_session', { session: newSession })
      sessions.value.push(result)
      return result
    } catch (err) {
      error.value = err as string
      throw err
    }
  }

  async function updateSession(session: Session) {
    try {
      const result = await invoke<Session>('update_session', { session })
      const index = sessions.value.findIndex(s => s.id === session.id)
      if (index !== -1) {
        sessions.value[index] = result
      }
      return result
    } catch (err) {
      error.value = err as string
      throw err
    }
  }

  async function deleteSession(sessionId: string) {
    try {
      await invoke('delete_session', { sessionId })
      sessions.value = sessions.value.filter(s => s.id !== sessionId)
      activeConnections.value.delete(sessionId)
    } catch (err) {
      error.value = err as string
      throw err
    }
  }

  async function connectToSession(sessionId: string) {
    try {
      const result = await invoke<string>('connect_ssh', { sessionId })
      console.log('Connection result:', result)
    } catch (err) {
      error.value = err as string
      throw err
    }
  }

  async function disconnectSession(sessionId: string) {
    try {
      await invoke('disconnect_session', { sessionId })
    } catch (err) {
      error.value = err as string
      throw err
    }
  }

  function updateConnectionStatus(status: ConnectionStatus) {
    activeConnections.value.set(status.session_id, status)
  }

  function getConnectionStatus(sessionId: string): ConnectionStatus | undefined {
    return activeConnections.value.get(sessionId)
  }

  // Initialize event listeners
  function initializeEventListeners() {
    listen<ConnectionStatus>('connection_status', (event) => {
      updateConnectionStatus(event.payload)
    })
  }

  return {
    // State
    sessions,
    activeConnections,
    isLoading,
    error,
    // Computed
    connectedSessions,
    // Actions
    loadSessions,
    createSession,
    updateSession,
    deleteSession,
    connectToSession,
    disconnectSession,
    updateConnectionStatus,
    getConnectionStatus,
    initializeEventListeners,
  }
})
