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
      
      // First try to load from persistent store
      const storedSessions = await invoke<Session[]>('load_sessions_from_store')
      sessions.value = storedSessions
      
      console.log('Loaded sessions from store:', storedSessions)
    } catch (err) {
      console.error('Failed to load sessions:', err)
      error.value = err as string
    } finally {
      isLoading.value = false
    }
  }

  async function saveSessions() {
    try {
      await invoke('save_sessions_to_store')
      console.log('Sessions saved to store')
    } catch (err) {
      console.error('Failed to save sessions:', err)
      error.value = err as string
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
      
      // Extract password and private_key_path from auth_method
      let password: string | undefined = undefined
      let private_key_path: string | undefined = undefined
      
      if (sessionData.auth_method === 'Password') {
        // For password auth, we'll pass undefined - password would need to be collected separately
        password = undefined
      } else if (typeof sessionData.auth_method === 'object' && 'PublicKey' in sessionData.auth_method) {
        private_key_path = sessionData.auth_method.PublicKey.key_path
      }
      
      console.log('Store: Creating session:', newSession)
      const result = await invoke<Session>('create_session', {
        name: newSession.name,
        host: newSession.host,
        port: newSession.port,
        username: newSession.username,
        password,
        private_key_path,
        protocol: newSession.protocol,
      })
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
      const result = await invoke<Session>('update_session', {
        session: {
          id: session.id,
          name: session.name,
          host: session.host,
          port: session.port,
          username: session.username,
          auth_method: session.auth_method,
          protocol: session.protocol,
          created_at: session.created_at,
          last_used: session.last_used,
        }
      })
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
      const session = sessions.value.find(s => s.id === sessionId)
      if (!session) {
        throw new Error('Session not found')
      }
      
      console.log('SessionStore: connectToSession called for:', sessionId, 'Auth method:', session.auth_method)
      
      // For password authentication, don't connect here - let Terminal component handle it
      if (session.auth_method === 'Password') {
        console.log('Password authentication detected - connection will be handled by Terminal component')
        return
      }
      
      // For non-password auth, use the old connection method
      console.log('Using old connection method for non-password auth')
      const result = await invoke<string>('connect_ssh', { sessionId })
      console.log('Connection result:', result)
    } catch (err) {
      error.value = err as string
      throw err
    }
  }

  async function disconnectSession(sessionId: string) {
    try {
      console.log('Store: Disconnecting session:', sessionId)
      console.log('Store: Current connection status before:', activeConnections.value.get(sessionId))
      
      await invoke('disconnect_session', { sessionId })
      console.log('Store: Backend disconnect call completed')
      
      // Manually update the connection status to ensure UI updates
      const newStatus = {
        session_id: sessionId,
        status: 'disconnected',
        message: 'Disconnected'
      }
      
      console.log('Store: Updating connection status to:', newStatus)
      updateConnectionStatus(newStatus)
      
      console.log('Store: Connection status after update:', activeConnections.value.get(sessionId))
      console.log('Store: Session disconnected successfully:', sessionId)
    } catch (err) {
      console.error('Store: Failed to disconnect session:', err)
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
    saveSessions,
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
