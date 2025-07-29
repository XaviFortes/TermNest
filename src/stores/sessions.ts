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
  const activeSession = ref<Session | null>(null)
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
      console.log('Store: Loading sessions...')
      const result = await invoke<Session[]>('list_sessions')
      console.log('Store: Loaded sessions:', result)
      sessions.value = result
    } catch (err) {
      console.error('Store: Failed to load sessions:', err)
      error.value = err as string
    } finally {
      isLoading.value = false
    }
  }

  async function createSession(sessionData: Omit<Session, 'id' | 'created_at'>) {
    try {
      // Generate a UUID, with fallback for environments where crypto.randomUUID is not available
      const generateId = () => {
        if (typeof crypto !== 'undefined' && crypto.randomUUID) {
          return crypto.randomUUID()
        }
        // Fallback UUID generation
        return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
          const r = Math.random() * 16 | 0
          const v = c === 'x' ? r : (r & 0x3 | 0x8)
          return v.toString(16)
        })
      }
      
      const newSession: Session = {
        ...sessionData,
        id: generateId(),
        created_at: new Date().toISOString(),
      }
      
      console.log('Store: Creating session:', newSession)
      const result = await invoke<Session>('create_session', { session: newSession })
      console.log('Store: Session created result:', result)
      sessions.value.push(result)
      return result
    } catch (err) {
      console.error('Store: Failed to create session:', err)
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

  function openSession(session: Session) {
    console.log('Opening session:', session)
    activeSession.value = session
  }

  function closeSession() {
    console.log('Closing active session')
    activeSession.value = null
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
    activeSession,
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
    openSession,
    closeSession,
    initializeEventListeners,
  }
})
