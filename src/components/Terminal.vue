<template>
  <div class="terminal-container">
    <div class="terminal-header">
      <div class="terminal-title">
        <span class="connection-indicator" :class="connectionStatus"></span>
        {{ sessionName }}
      </div>
      <div class="terminal-actions">
        <button @click="toggleSftp" v-if="protocol === 'SSH'" class="btn btn-sm">
          SFTP
        </button>
        <button @click="disconnect" class="btn btn-sm btn-danger">
          Disconnect
        </button>
      </div>
    </div>
    
    <div class="terminal-body">
      <div class="terminal-wrapper" :class="{ 'full-width': !showSftp }">
        <!-- Connection Log (only shown when connecting) -->
        <div v-if="connectionStatus === 'connecting'" class="connection-log">
          <div class="connection-log-header">
            <div class="connection-spinner"></div>
            <h4>Establishing SSH Connection</h4>
          </div>
          <div class="connection-steps">
            <div 
              v-for="(step, index) in connectionSteps" 
              :key="index"
              class="connection-step"
              :class="{ 'active': index === currentStepIndex, 'completed': index < currentStepIndex }"
            >
              <div class="step-indicator">
                <span v-if="index < currentStepIndex" class="step-check">✓</span>
                <span v-else-if="index === currentStepIndex" class="step-spinner">⟳</span>
                <span v-else class="step-pending">○</span>
              </div>
              <div class="step-text">{{ step }}</div>
            </div>
          </div>
          <div v-if="currentStepMessage" class="current-step-message">
            {{ currentStepMessage }}
          </div>
        </div>

        <!-- Terminal Content -->
        <div 
          ref="terminalContainer" 
          class="terminal-content"
          :class="{ 'with-connection-log': connectionStatus === 'connecting' }"
          @click="focusTerminal"
        >
          <!-- XTerm.js will be mounted here -->
        </div>
      </div>
      
      <!-- SFTP Panel -->
      <div v-if="showSftp" class="sftp-panel" :class="{ compact: !sftpExpanded }">
        <div class="sftp-header">
          <div class="sftp-title">
            <h3>📁 Files</h3>
            <span v-if="sftpExpanded" class="file-count">{{ files.length }} items</span>
          </div>
          <div class="sftp-actions">
            <button 
              @click="sftpExpanded = !sftpExpanded" 
              class="btn btn-sm btn-expand" 
              :title="sftpExpanded ? 'Compact view' : 'Expanded view'"
            >
              {{ sftpExpanded ? '⬅️' : '➡️' }}
            </button>
            <button @click="refreshFiles" class="btn btn-sm btn-refresh" title="Refresh">
              🔄
            </button>
            <button @click="closeSftp" class="btn btn-sm btn-close" title="Close">×</button>
          </div>
        </div>
        
        <div class="sftp-content">
          <div class="file-browser">
            <!-- Compact Path Navigation -->
            <div class="path-bar" :class="{ compact: !sftpExpanded }">
              <div v-if="sftpExpanded" class="path-breadcrumb">
                <span v-for="(segment, index) in pathSegments" :key="index" class="breadcrumb-item">
                  <button 
                    @click="navigateToSegment(index)" 
                    class="breadcrumb-btn"
                    :class="{ active: index === pathSegments.length - 1 }"
                  >
                    {{ segment || '/' }}
                  </button>
                  <span v-if="index < pathSegments.length - 1" class="breadcrumb-separator">/</span>
                </span>
              </div>
              
              <div class="path-controls" :class="{ compact: !sftpExpanded }">
                <input 
                  v-model="currentPath" 
                  @keyup.enter="navigateToPath" 
                  class="path-input" 
                  :placeholder="sftpExpanded ? 'Enter path...' : currentPath"
                  :title="currentPath"
                />
                <button @click="navigateUp" class="btn btn-sm btn-up" title="Go up" :disabled="currentPath === '/'">
                  ⬆️
                </button>
              </div>
            </div>
            
            <!-- File List -->
            <div class="file-list-container">
              <div v-if="files.length === 0" class="empty-state">
                <div class="empty-icon">📂</div>
                <div v-if="sftpExpanded" class="empty-text">No files found</div>
                <button @click="refreshFiles" class="btn btn-sm">Refresh</button>
              </div>
              
              <div v-else class="file-list">
                <!-- Expanded Header -->
                <div v-if="sftpExpanded" class="file-list-header">
                  <div class="col-name">Name</div>
                  <div class="col-size">Size</div>
                  <div class="col-modified">Modified</div>
                  <div class="col-actions">Actions</div>
                </div>
                
                <div class="file-list-body">
                  <div 
                    v-for="file in files" 
                    :key="file.path"
                    class="file-item"
                    :class="{ 
                      'is-directory': file.is_directory,
                      'is-parent': file.name === '..',
                      'compact': !sftpExpanded,
                      'selected': selectedFile === file
                    }"
                    @dblclick="handleFileAction(file)"
                    @click="selectFile(file)"
                    @contextmenu.prevent="showFileContextMenu($event, file)"
                    @mouseenter="showTooltip"
                    @mouseleave="hideTooltip"
                    :title="getFileTooltip(file)"
                  >
                    <!-- Compact View -->
                    <div v-if="!sftpExpanded" class="file-compact">
                      <span class="file-icon">{{ getFileIcon(file) }}</span>
                      <span class="file-name">{{ file.name }}</span>
                    </div>
                    
                    <!-- Expanded View -->
                    <template v-else>
                      <div class="file-info">
                        <span class="file-icon">{{ getFileIcon(file) }}</span>
                        <span class="file-name">{{ file.name }}</span>
                      </div>
                      <div class="file-size">
                        {{ file.is_directory ? '-' : formatFileSize(file.size) }}
                      </div>
                      <div class="file-modified">
                        {{ file.modified || '-' }}
                      </div>
                      <div class="file-actions">
                        <button 
                          v-if="!file.is_directory && file.name !== '..'" 
                          @click.stop="downloadFile(file)" 
                          class="btn btn-xs btn-download"
                          title="Download file"
                        >
                          ⬇️
                        </button>
                        <button 
                          v-if="file.name !== '..'" 
                          @click.stop="deleteFile(file)" 
                          class="btn btn-xs btn-danger"
                          title="Delete"
                        >
                          🗑️
                        </button>
                      </div>
                    </template>
                  </div>
                </div>
              </div>
            </div>
            
            <!-- Compact Status Bar -->
            <div v-if="sftpExpanded" class="sftp-status">
              <div class="status-info">
                <span class="current-path">{{ currentPath }}</span>
              </div>
              <div class="status-actions">
                <span class="connection-status">
                  {{ connectionStatus === 'connected' ? '🟢 Connected' : '🔴 Disconnected' }}
                </span>
              </div>
            </div>
          </div>
        </div>
        
        <!-- Context Menu -->
        <div 
          v-if="showContextMenu" 
          class="context-menu"
          :style="{ 
            left: contextMenuPosition.x + 'px', 
            top: contextMenuPosition.y + 'px' 
          }"
          @click.stop
        >
          <div v-if="contextMenuFile" class="context-menu-items">
            <div class="context-menu-item" @click="handleFileAction(contextMenuFile!)">
              <span class="menu-icon">{{ contextMenuFile.is_directory ? '📂' : '👁️' }}</span>
              {{ contextMenuFile.is_directory ? 'Open' : 'View' }}
            </div>
            
            <div 
              v-if="!contextMenuFile.is_directory && contextMenuFile.name !== '..'" 
              class="context-menu-item" 
              @click="downloadFile(contextMenuFile!)"
            >
              <span class="menu-icon">⬇️</span>
              Download
            </div>
            
            <div 
              v-if="contextMenuFile.name !== '..'" 
              class="context-menu-item" 
              @click="renameFile(contextMenuFile!)"
            >
              <span class="menu-icon">✏️</span>
              Rename
            </div>
            
            <div class="context-menu-separator"></div>
            
            <div class="context-menu-item info-item">
              <div class="file-details">
                <div class="detail-row">
                  <span class="detail-label">Size:</span>
                  <span class="detail-value">{{ contextMenuFile.is_directory ? 'Directory' : formatFileSize(contextMenuFile.size) }}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">Modified:</span>
                  <span class="detail-value">{{ contextMenuFile.modified || 'Unknown' }}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">Path:</span>
                  <span class="detail-value">{{ contextMenuFile.path }}</span>
                </div>
              </div>
            </div>
            
            <div class="context-menu-separator"></div>
            
            <div 
              v-if="contextMenuFile.name !== '..'" 
              class="context-menu-item danger" 
              @click="deleteFile(contextMenuFile!)"
            >
              <span class="menu-icon">🗑️</span>
              Delete
            </div>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Password Dialog -->
    <PasswordDialog
      v-if="showPasswordDialog && pendingSessionConfig"
      :host="pendingSessionConfig.host"
      :username="pendingSessionConfig.username"
      :error="passwordDialogError"
      :is-authenticating="isAuthenticating"
      @authenticate="handlePasswordAuthentication"
      @cancel="handlePasswordCancel"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useSessionsStore } from '../stores/sessions'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import PasswordDialog from './PasswordDialog.vue'
import '@xterm/xterm/css/xterm.css'

interface Props {
  sessionId: string
  sessionName: string
  protocol: string
}

interface FileItem {
  name: string
  path: string
  size: number
  is_directory: boolean
  modified: string
}

const props = defineProps<Props>()
const sessionsStore = useSessionsStore()

// Terminal state
const terminalContainer = ref<HTMLElement>()
const connectionStatus = ref<'connecting' | 'connected' | 'disconnected'>('connecting')
let terminal: Terminal | null = null
let fitAddon: FitAddon | null = null

// Connection log state
const connectionSteps = ref([
  'Establishing TCP connection',
  'Creating SSH session',
  'Performing SSH handshake',
  'Authenticating user',
  'Authentication successful',
  'Creating terminal session',
  'Starting remote shell',
  'Setting up terminal I/O',
  'Connection established'
])
const currentStepIndex = ref(0)
const currentStepMessage = ref('')

// SFTP state
const showSftp = ref(false)
const currentPath = ref('/')
const files = ref<FileItem[]>([])
const selectedFile = ref<FileItem | null>(null)
const sftpExpanded = ref(false)
const showContextMenu = ref(false)
const contextMenuPosition = ref({ x: 0, y: 0 })
const contextMenuFile = ref<FileItem | null>(null)

// Password dialog state
const showPasswordDialog = ref(false)
const passwordDialogError = ref('')
const isAuthenticating = ref(false)
const pendingSessionConfig = ref<any>(null)

// SFTP password handling
const pendingPasswordResolve = ref<((password: string) => void) | null>(null)
const pendingPasswordReject = ref<((error: Error) => void) | null>(null)
const storedSessionPassword = ref<string | null>(null) // Store password for SFTP operations

// Event listener cleanup
let unlistenTerminalOutput: (() => void) | null = null
let unlistenConnectionStatus: (() => void) | null = null

// Initialize terminal
onMounted(async () => {
  setupTerminal()
  await setupEventListeners()  // Set up listeners BEFORE connecting
  await initializeTerminal()
  focusTerminal()
})

onUnmounted(() => {
  disconnect()
  if (terminal) {
    terminal.dispose()
  }
  if (unlistenTerminalOutput) {
    unlistenTerminalOutput()
  }
  if (unlistenConnectionStatus) {
    unlistenConnectionStatus()
  }
  
  // Hide context menu on unmount
  hideContextMenu()
  
  // Cleanup terminal resize handlers
  if (terminalContainer.value && (terminalContainer.value as any).__terminalCleanup) {
    ;(terminalContainer.value as any).__terminalCleanup()
  }
})

async function setupEventListeners() {
  // Listen for terminal output from the new SSH backend
  unlistenTerminalOutput = await listen('terminal-data', (event: any) => {
    const payload = event.payload
    if (payload.session_id === props.sessionId) {
      appendOutput(payload.data)
    }
  })

  // Listen for connection status updates
  unlistenConnectionStatus = await listen('connection_status', (event: any) => {
    const payload = event.payload
    if (payload.session_id === props.sessionId) {
      updateConnectionProgress(payload.status, payload.message)
    }
  })
}

function updateConnectionProgress(status: string, message?: string) {
  currentStepMessage.value = message || ''
  
  if (status === 'connecting' && message) {
    // Map status messages to step indices
    const stepMap: { [key: string]: number } = {
      'Establishing TCP connection': 0,
      'Creating SSH session': 1,
      'Performing SSH handshake': 2,
      'Authenticating user': 3,
      'Authentication successful': 4,
      'Creating terminal session': 5,
      'Starting remote shell': 6,
      'Setting up terminal I/O': 7
    }
    
    // Find matching step or advance by keywords
    for (const [keyword, index] of Object.entries(stepMap)) {
      if (message.includes(keyword) || message.toLowerCase().includes(keyword.toLowerCase())) {
        currentStepIndex.value = index
        break
      }
    }
    
    // Handle specific cases
    if (message.includes('TCP connection')) currentStepIndex.value = 0
    else if (message.includes('SSH session')) currentStepIndex.value = 1
    else if (message.includes('handshake')) currentStepIndex.value = 2
    else if (message.includes('Authenticating')) currentStepIndex.value = 3
    else if (message.includes('Authentication successful')) currentStepIndex.value = 4
    else if (message.includes('terminal session')) currentStepIndex.value = 5
    else if (message.includes('remote shell')) currentStepIndex.value = 6
    else if (message.includes('terminal I/O')) currentStepIndex.value = 7
  } else if (status === 'connected') {
    currentStepIndex.value = connectionSteps.value.length - 1
    connectionStatus.value = 'connected'
    sessionsStore.updateConnectionStatus({
      session_id: props.sessionId,
      status: 'connected',
      message: 'Connection established'
    })
  } else if (status === 'disconnected') {
    connectionStatus.value = 'disconnected'
    sessionsStore.updateConnectionStatus({
      session_id: props.sessionId,
      status: 'disconnected',
      message: 'Connection closed'
    })
  }
}

async function initializeTerminal() {
  try {
    connectionStatus.value = 'connecting'
    
    // For additional connections, extract the original session ID
    const originalSessionId = props.sessionId.includes('_conn_') 
      ? props.sessionId.split('_conn_')[0] 
      : props.sessionId
    
    const session = sessionsStore.sessions.find(s => s.id === originalSessionId)
    if (!session) {
      throw new Error('Session not found')
    }
    
    // Check if this is password auth - always prompt for password
    if (session.auth_method === 'Password') {
      console.log('Password authentication detected in Terminal - showing password dialog')
      // Store basic config for password dialog
      pendingSessionConfig.value = {
        host: session.host,
        port: session.port,
        username: session.username,
        auth_method: 'Password'
      }
      showPasswordDialog.value = true
      return
    }
    
    // For non-password auth, proceed directly
    const sessionConfig = await getSessionConfig()
    await invoke('ssh_connect', {
      sessionId: props.sessionId,
      config: sessionConfig
    })
    
    connectionStatus.value = 'connected'
    sessionsStore.updateConnectionStatus({
      session_id: props.sessionId,
      status: 'connected',
      message: 'SSH connection established'
    })
    
    if (props.protocol === 'SSH') {
      await loadRemoteFiles()
    }
  } catch (error) {
    console.error('Failed to initialize terminal:', error)
    connectionStatus.value = 'disconnected'
    sessionsStore.updateConnectionStatus({
      session_id: props.sessionId,
      status: 'disconnected',
      message: 'Failed to connect: ' + error
    })
    if (terminal) {
      terminal.write('Failed to connect: ' + error + '\r\n')
    }
  }
}

async function getSessionConfig() {
  // For additional connections, extract the original session ID
  const originalSessionId = props.sessionId.includes('_conn_') 
    ? props.sessionId.split('_conn_')[0] 
    : props.sessionId
    
  const session = sessionsStore.sessions.find(s => s.id === originalSessionId)
  if (!session) {
    throw new Error('Session not found')
  }
  
  const baseConfig = {
    host: session.host,
    port: session.port,
    username: session.username
  }
  
  if (session.auth_method === 'Password') {
    throw new Error('Password authentication requires user input')
  } else if (typeof session.auth_method === 'object' && 'PublicKey' in session.auth_method) {
    return {
      ...baseConfig,
      auth_method: { PublicKey: { private_key_path: session.auth_method.PublicKey.key_path } }
    }
  } else if (session.auth_method === 'Agent') {
    return {
      ...baseConfig,
      auth_method: 'Agent'
    }
  } else {
    throw new Error('Unsupported authentication method')
  }
}

function appendOutput(text: string) {
  if (terminal) {
    terminal.write(text)
  }
}

function setupTerminal() {
  if (!terminalContainer.value) return
  
  // Create terminal instance
  terminal = new Terminal({
    theme: {
      background: '#1a1a1a',
      foreground: '#ffffff',
      cursor: '#ffffff',
      selectionBackground: '#3e3e3e',
      black: '#000000',
      red: '#ff5555',
      green: '#50fa7b',
      yellow: '#f1fa8c',
      blue: '#bd93f9',
      magenta: '#ff79c6',
      cyan: '#8be9fd',
      white: '#ffffff',
      brightBlack: '#44475a',
      brightRed: '#ff5555',
      brightGreen: '#50fa7b',
      brightYellow: '#f1fa8c',
      brightBlue: '#bd93f9',
      brightMagenta: '#ff79c6',
      brightCyan: '#8be9fd',
      brightWhite: '#ffffff'
    },
    fontSize: 12, /* Reduced from 14 */
    fontFamily: 'Monaco, Menlo, "Ubuntu Mono", monospace',
    cursorBlink: true,
    convertEol: true,
    scrollback: 10000, /* Increase scrollback for better history */
    fastScrollModifier: 'alt', /* Enable fast scrolling */
    allowTransparency: false,
    smoothScrollDuration: 100
  })
  
  // Create fit addon
  fitAddon = new FitAddon()
  terminal.loadAddon(fitAddon)
  
  // Open terminal in container
  terminal.open(terminalContainer.value)
  
  // Fit terminal to container size
  setTimeout(() => {
    if (fitAddon) {
      fitAddon.fit()
    }
  }, 0)
  
  // Handle user input
  terminal.onData((data) => {
    // Do NOT echo locally - let SSH server handle echo
    // Send raw keypress data without conversion to preserve terminal semantics
    
    // Forward keystrokes to SSH backend using new API
    invoke('ssh_send_input', {
      sessionId: props.sessionId,
      input: data
    }).catch(console.error)
  })

  
  // Handle resize with debouncing - just fit the terminal to container
  let resizeTimeout: number | null = null
  const resizeObserver = new ResizeObserver(() => {
    if (resizeTimeout) {
      clearTimeout(resizeTimeout)
    }
    resizeTimeout = window.setTimeout(async () => {
      if (fitAddon && terminal) {
        try {
          fitAddon.fit()
          console.log(`Terminal fitted to container: ${terminal.cols}x${terminal.rows}`)
        } catch (error) {
          console.warn('Terminal resize failed:', error)
        }
      }
    }, 100) // Increased debounce time for better performance
  })
  
  resizeObserver.observe(terminalContainer.value)
  
  // Also handle window resize
  const handleWindowResize = async () => {
    if (resizeTimeout) {
      clearTimeout(resizeTimeout)
    }
    resizeTimeout = window.setTimeout(async () => {
      if (fitAddon && terminal) {
        try {
          fitAddon.fit()
          
          // Get terminal dimensions and notify backend
          const cols = terminal.cols
          const rows = terminal.rows
          
          // Send resize to backend
          try {
            await invoke('ssh_resize_terminal', {
              sessionId: props.sessionId,
              cols: cols,
              rows: rows
            })
            console.log(`Terminal window resized to ${cols}x${rows}`)
          } catch (error) {
            console.warn('Failed to resize backend terminal on window resize:', error)
          }
        } catch (error) {
          console.warn('Terminal window resize failed:', error)
        }
      }
    }, 150) // Slightly longer delay for window resize
  }
  
  window.addEventListener('resize', handleWindowResize)
  
  // Cleanup function
  const cleanup = () => {
    if (resizeTimeout) {
      clearTimeout(resizeTimeout)
    }
    window.removeEventListener('resize', handleWindowResize)
    resizeObserver.disconnect()
  }
  
  // Store cleanup function for later use
  ;(terminalContainer.value as any).__terminalCleanup = cleanup
}

function focusTerminal() {
  if (terminal) {
    terminal.focus()
  }
}

// SFTP functions
function toggleSftp() {
  showSftp.value = !showSftp.value
  if (showSftp.value) {
    refreshFiles()
  }
  
  // Resize terminal after SFTP panel toggle
  setTimeout(() => {
    if (fitAddon && terminal) {
      fitAddon.fit()
    }
  }, 300) // Wait for animation to complete
}

function closeSftp() {
  showSftp.value = false
  hideContextMenu()
  
  // Resize terminal after closing SFTP
  setTimeout(() => {
    if (fitAddon && terminal) {
      fitAddon.fit()
    }
  }, 100)
}

async function loadRemoteFiles() {
  try {
    let result;
    
    if (isPasswordSession()) {
      // Use stored password if available, otherwise prompt
      const password = storedSessionPassword.value || await promptForPassword()
      result = await invoke('list_remote_directory_with_password', {
        sessionId: props.sessionId,
        path: currentPath.value,
        password: password
      })
    } else {
      // Use regular command for key-based authentication
      result = await invoke('list_remote_directory', {
        sessionId: props.sessionId,
        path: currentPath.value
      })
    }
    
    files.value = result as FileItem[]
  } catch (error) {
    console.error('Failed to load remote files:', error)
    // Simulate some files
    files.value = [
      { name: '..', path: '/', size: 0, is_directory: true, modified: '' },
      { name: 'documents', path: '/documents', size: 0, is_directory: true, modified: '2024-01-15' },
      { name: 'config.txt', path: '/config.txt', size: 1024, is_directory: false, modified: '2024-01-15' },
      { name: 'script.sh', path: '/script.sh', size: 2048, is_directory: false, modified: '2024-01-14' }
    ]
  }
}

function refreshFiles() {
  loadRemoteFiles()
}

function navigateToPath() {
  loadRemoteFiles()
}

function handleFileAction(file: FileItem) {
  hideContextMenu()
  if (file.is_directory) {
    currentPath.value = file.path
    navigateToPath()
  }
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

// Computed property for path breadcrumbs
const pathSegments = computed(() => {
  if (currentPath.value === '/') return ['/']
  return currentPath.value.split('/').filter(segment => segment !== '')
})

// Enhanced file browser functions
function navigateToSegment(index: number) {
  if (index === 0 && pathSegments.value[0] === '/') {
    currentPath.value = '/'
  } else {
    currentPath.value = '/' + pathSegments.value.slice(0, index + 1).join('/')
  }
  navigateToPath()
}

function navigateUp() {
  if (currentPath.value === '/') return
  const parentPath = currentPath.value.substring(0, currentPath.value.lastIndexOf('/'))
  currentPath.value = parentPath || '/'
  navigateToPath()
}

function selectFile(file: FileItem) {
  selectedFile.value = file
}

function getFileIcon(file: FileItem): string {
  if (file.name === '..') return '⬅️'
  if (file.is_directory) return '📁'
  
  // Determine icon based on file extension
  const ext = file.name.split('.').pop()?.toLowerCase()
  switch (ext) {
    case 'txt': case 'md': case 'log': return '📄'
    case 'js': case 'ts': case 'json': return '📜'
    case 'html': case 'css': case 'xml': return '🌐'
    case 'py': case 'rb': case 'php': return '🐍'
    case 'jpg': case 'jpeg': case 'png': case 'gif': case 'svg': return '🖼️'
    case 'pdf': return '📕'
    case 'zip': case 'tar': case 'gz': case '7z': return '📦'
    case 'mp3': case 'wav': case 'flac': return '🎵'
    case 'mp4': case 'avi': case 'mkv': return '🎬'
    case 'sh': case 'bash': case 'zsh': return '⚡'
    case 'conf': case 'cfg': case 'ini': return '⚙️'
    default: return '📄'
  }
}

// Context menu functions
function showFileContextMenu(event: MouseEvent, file: FileItem) {
  event.preventDefault()
  contextMenuFile.value = file
  
  // Calculate position to prevent menu from going off-screen
  const menuWidth = 200 // Approximate width of context menu
  const menuHeight = 200 // Approximate height of context menu
  const viewportWidth = window.innerWidth
  const viewportHeight = window.innerHeight
  
  let x = event.clientX
  let y = event.clientY
  
  // Adjust x position if menu would go off right edge
  if (x + menuWidth > viewportWidth) {
    x = viewportWidth - menuWidth - 10
  }
  
  // Adjust y position if menu would go off bottom edge
  if (y + menuHeight > viewportHeight) {
    y = viewportHeight - menuHeight - 10
  }
  
  // Ensure menu doesn't go off top or left edges
  x = Math.max(5, x)
  y = Math.max(5, y)
  
  contextMenuPosition.value = { x, y }
  showContextMenu.value = true
  
  // Close context menu when clicking elsewhere
  setTimeout(() => {
    document.addEventListener('click', hideContextMenu, { once: true })
  })
}

function hideContextMenu() {
  showContextMenu.value = false
  contextMenuFile.value = null
}

function getFileTooltip(file: FileItem): string {
  if (!sftpExpanded.value) {
    return `${file.name}\nSize: ${file.is_directory ? 'Directory' : formatFileSize(file.size)}\nModified: ${file.modified || 'Unknown'}`
  }
  return file.path
}

function showTooltip() {
  // Tooltip functionality can be enhanced later if needed
}

function hideTooltip() {
  // Tooltip functionality can be enhanced later if needed
}

function renameFile(file: FileItem) {
  const newName = prompt('Enter new name:', file.name)
  if (newName && newName !== file.name) {
    // TODO: Implement rename functionality
    appendOutput(`Rename functionality not yet implemented for: ${file.name} -> ${newName}\n`)
  }
  hideContextMenu()
}

// Helper function to get session config for SFTP operations
function getSftpSessionConfig() {
  // For additional connections, extract the original session ID
  const originalSessionId = props.sessionId.includes('_conn_') 
    ? props.sessionId.split('_conn_')[0] 
    : props.sessionId
    
  const session = sessionsStore.sessions.find(s => s.id === originalSessionId)
  if (!session) {
    throw new Error('Session not found')
  }
  
  return session
}

// Helper function to check if session uses password authentication
function isPasswordSession(): boolean {
  try {
    const session = getSftpSessionConfig()
    return session.auth_method === 'Password'
  } catch {
    return false
  }
}

// Helper function to prompt for password
function promptForPassword(): Promise<string> {
  return new Promise((resolve, reject) => {
    const cleanup = () => {
      showPasswordDialog.value = false
      passwordDialogError.value = ''
    }

    const handlePasswordSubmit = (password: string) => {
      cleanup()
      resolve(password)
    }

    const handlePasswordCancel = () => {
      cleanup()
      reject(new Error('Password entry cancelled'))
    }

    // Set up the dialog
    pendingPasswordResolve.value = handlePasswordSubmit
    pendingPasswordReject.value = handlePasswordCancel
    showPasswordDialog.value = true
  })
}

async function downloadFile(file: FileItem) {
  hideContextMenu()
  try {
    appendOutput('Downloading ' + file.name + '...\n')
    
    // For now, download to a simple path (user can specify where later)
    const downloadsPath = `./downloads/${file.name}`;
    
    let result;
    
    if (isPasswordSession()) {
      // Use stored password if available, otherwise prompt
      const password = storedSessionPassword.value || await promptForPassword()
      result = await invoke('download_remote_file_with_password', {
        sessionId: props.sessionId,
        remotePath: file.path,
        localPath: downloadsPath,
        password: password
      })
    } else {
      // Use regular command for key-based authentication
      result = await invoke('download_remote_file', {
        sessionId: props.sessionId,
        remotePath: file.path,
        localPath: downloadsPath
      })
    }
    
    appendOutput(result + '\n\n')
  } catch (error) {
    appendOutput('Download failed: ' + error + '\n\n')
  }
}

async function deleteFile(file: FileItem) {
  hideContextMenu()
  if (confirm('Are you sure you want to delete ' + file.name + '?')) {
    try {
      appendOutput('Deleting ' + file.name + '...\n')
      
      let result;
      
      if (isPasswordSession()) {
        // Use stored password if available, otherwise prompt
        const password = storedSessionPassword.value || await promptForPassword()
        result = await invoke('delete_remote_file_with_password', {
          sessionId: props.sessionId,
          remotePath: file.path,
          password: password
        })
      } else {
        // Use regular command for key-based authentication
        result = await invoke('delete_remote_file', {
          sessionId: props.sessionId,
          remotePath: file.path
        })
      }
      
      appendOutput(result + '\n\n')
      refreshFiles()
    } catch (error) {
      appendOutput('Delete failed: ' + error + '\n\n')
    }
  }
}

// Password authentication handlers
async function handlePasswordAuthentication(password: string) {
  // Check if this is for SFTP operation
  if (pendingPasswordResolve.value) {
    pendingPasswordResolve.value(password)
    return
  }
  
  // Otherwise, handle SSH authentication
  if (!pendingSessionConfig.value) return
  
  isAuthenticating.value = true
  passwordDialogError.value = ''
  
  try {
    // Create the SSH config with the provided password
    const config = {
      host: pendingSessionConfig.value.host,
      port: pendingSessionConfig.value.port,
      username: pendingSessionConfig.value.username,
      auth_method: { Password: { password: password } }
    }
    
    await invoke('ssh_connect_with_password', {
      sessionId: props.sessionId,
      config: config,
      password: password
    })
    
    // Store password for SFTP operations
    storedSessionPassword.value = password
    
    // Success - close dialog and update status
    showPasswordDialog.value = false
    connectionStatus.value = 'connected'
    sessionsStore.updateConnectionStatus({
      session_id: props.sessionId,
      status: 'connected',
      message: 'SSH connection established via password'
    })
    isAuthenticating.value = false
    pendingSessionConfig.value = null
    
    if (props.protocol === 'SSH') {
      await loadRemoteFiles()
    }
  } catch (error) {
    console.error('Password authentication failed:', error)
    passwordDialogError.value = 'Authentication failed: ' + error
    isAuthenticating.value = false
  }
}

function handlePasswordCancel() {
  // Check if this is for SFTP operation
  if (pendingPasswordReject.value) {
    pendingPasswordReject.value(new Error('Password entry cancelled'))
    return
  }
  
  // Otherwise, handle SSH authentication cancellation
  showPasswordDialog.value = false
  connectionStatus.value = 'disconnected'
  sessionsStore.updateConnectionStatus({
    session_id: props.sessionId,
    status: 'disconnected',
    message: 'Password authentication cancelled'
  })
  pendingSessionConfig.value = null
  passwordDialogError.value = ''
  isAuthenticating.value = false
}

function disconnect() {
  console.log('Disconnecting session:', props.sessionId)
  
  // Clear stored password for security
  storedSessionPassword.value = null
  
  // Disconnect from SSH backend using new API
  invoke('ssh_disconnect', { sessionId: props.sessionId })
    .then(() => {
      console.log('Session disconnected successfully')
    })
    .catch((error) => {
      console.error('Failed to disconnect session:', error)
    })
  
  // Update local state
  connectionStatus.value = 'disconnected'
  sessionsStore.updateConnectionStatus({
    session_id: props.sessionId,
    status: 'disconnected',
    message: 'Session disconnected'
  })
  
  // Close the session and go back to sessions list
  sessionsStore.closeSession()
}
</script>

<style scoped>
.terminal-container {
  height: calc(100vh - 40px); /* Account for window titlebar and borders */
  display: flex;
  flex-direction: column;
  background: #1e1e1e;
  color: #ffffff;
  font-family: 'Monaco', 'Menlo', 'Consolas', monospace;
  overflow: hidden;
}

.terminal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 0.75rem; /* Reduced from 0.75rem 1rem */
  background: #2d2d2d;
  border-bottom: 1px solid #404040;
  flex-shrink: 0;
}

.terminal-body {
  flex: 1;
  display: flex;
  overflow: hidden;
  min-height: 0;
  height: calc(100vh - 120px); /* Account for titlebar + header + some padding */
}

.terminal-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 200px;
  /* Remove max-width constraint - let it use full space when SFTP is closed */
}

.terminal-wrapper.full-width {
  width: 100%;
  max-width: 100%;
}

.terminal-title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-weight: 600;
}

.connection-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  display: inline-block;
}

.connection-indicator.connecting {
  background: #ffc107;
  animation: pulse 1s infinite;
}

.connection-indicator.connected {
  background: #28a745;
}

.connection-indicator.disconnected {
  background: #dc3545;
}

.terminal-actions {
  display: flex;
  gap: 0.5rem;
}

.terminal-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0; /* Important for flex child */
}

/* Connection Log Styles */
.connection-log {
  background: #2a2a2a;
  border-bottom: 1px solid #404040;
  padding: 1.5rem;
  max-height: 300px;
  overflow-y: auto;
}

.connection-log-header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 1.5rem;
}

.connection-log-header h4 {
  margin: 0;
  color: #ffffff;
  font-size: 1.1rem;
  font-weight: 600;
}

.connection-spinner {
  width: 20px;
  height: 20px;
  border: 2px solid #404040;
  border-top: 2px solid #007acc;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.connection-steps {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.connection-step {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.5rem;
  border-radius: 6px;
  transition: all 0.3s ease;
}

.connection-step.active {
  background: rgba(0, 122, 204, 0.1);
  border-left: 3px solid #007acc;
}

.connection-step.completed {
  opacity: 0.7;
}

.step-indicator {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  font-weight: bold;
  font-size: 12px;
}

.step-check {
  background: #28a745;
  color: white;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
}

.step-spinner {
  color: #007acc;
  animation: rotate 1s linear infinite;
}

@keyframes rotate {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.step-pending {
  color: #666;
  border: 2px solid #666;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
}

.step-text {
  color: #ffffff;
  font-size: 0.9rem;
}

.current-step-message {
  margin-top: 1rem;
  padding: 0.75rem;
  background: rgba(0, 122, 204, 0.1);
  border-radius: 6px;
  color: #007acc;
  font-size: 0.85rem;
  font-style: italic;
}

.terminal-content {
  flex: 1;
  overflow: hidden; /* Let xterm.js handle scrolling */
  cursor: text;
  min-height: 0; /* Important for flex child */
  height: 100%; /* Ensure full height usage */
  max-height: calc(100vh - 160px); /* Reserve space for header and controls, prevent bottom overflow */
}

.terminal-simulation {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.terminal-prompt {
  color: #00ff00;
  margin-bottom: 0.25rem;
}

.terminal-input {
  background: transparent;
  border: none;
  outline: none;
  color: #ffffff;
  font-family: inherit;
  min-height: 1.2em;
  margin-bottom: 0.5rem;
}

.terminal-input:empty:before {
  content: attr(data-placeholder);
  color: #666;
}

.terminal-output {
  flex: 1;
  white-space: pre-wrap;
  line-height: 1.4;
}

.sftp-panel {
  width: 350px;
  min-width: 250px;
  max-width: min(450px, 40vw); /* Responsive width based on viewport */
  background: #2d2d2d;
  border-left: 2px solid #404040;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  height: 100%;
  overflow: hidden;
  box-shadow: -2px 0 8px rgba(0, 0, 0, 0.3);
  animation: slideIn 0.3s ease-out;
  transition: width 0.3s ease;
}

.sftp-panel.compact {
  width: 200px;
  min-width: 180px;
  max-width: min(250px, 30vw); /* Responsive compact width */
}

@keyframes slideIn {
  from {
    width: 0;
    opacity: 0;
  }
  to {
    width: 350px;
    opacity: 1;
  }
}

.sftp-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 0.75rem; /* Reduced from 0.75rem 1rem */
  background: #404040;
  border-bottom: 1px solid #555;
  flex-shrink: 0;
}

.sftp-title {
  display: flex;
  flex-direction: column;
  gap: 0.15rem; /* Reduced from 0.2rem */
}

.sftp-title h3 {
  margin: 0;
  font-size: 0.8125rem; /* Reduced from 0.875rem */
  font-weight: 600;
}

.file-count {
  font-size: 0.625rem; /* Reduced from 0.7rem */
  color: #888;
}

.sftp-actions {
  display: flex;
  gap: 0.375rem; /* Reduced from 0.5rem */
}

.sftp-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.file-browser {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.path-bar {
  padding: 0.75rem;
  background: #353535;
  border-bottom: 1px solid #555;
  flex-shrink: 0;
}

.path-bar.compact {
  padding: 0.5rem;
}

.path-controls.compact {
  flex-direction: column;
  gap: 0.25rem;
}

.path-controls.compact .path-input {
  font-size: 0.7rem;
  padding: 0.2rem 0.4rem;
}

.file-item.compact {
  display: block;
  grid-template-columns: none;
  padding: 0.3rem 0.5rem; /* Reduced from 0.4rem 0.6rem */
}

.file-item.compact:hover {
  background: #404040;
}

.file-item.selected {
  background: #0d6efd;
  color: white;
}

.file-compact {
  display: flex;
  align-items: center;
  gap: 0.375rem; /* Reduced from 0.5rem */
}

.file-compact .file-icon {
  width: 14px; /* Reduced from 16px */
  text-align: center;
  flex-shrink: 0;
}

.file-compact .file-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  font-size: 0.75rem;
}

/* Context Menu */
.context-menu {
  position: fixed;
  background: #2d2d2d;
  border: 1px solid #555;
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  z-index: 9999;
  min-width: 200px;
  max-width: 300px;
  padding: 0.5rem 0;
}

.context-menu-items {
  display: flex;
  flex-direction: column;
}

.context-menu-item {
  display: flex;
  align-items: center;
  gap: 0.5rem; /* Reduced from 0.75rem */
  padding: 0.375rem 0.75rem; /* Reduced from 0.5rem 1rem */
  cursor: pointer;
  font-size: 0.75rem; /* Reduced from 0.8rem */
  transition: background-color 0.2s;
}

.context-menu-item:hover {
  background: #404040;
}

.context-menu-item.danger:hover {
  background: #dc3545;
}

.context-menu-item.info-item {
  cursor: default;
  padding: 0.5rem 0.75rem; /* Reduced from 0.75rem 1rem */
}

.context-menu-item.info-item:hover {
  background: #353535;
}

.menu-icon {
  width: 14px; /* Reduced from 16px */
  text-align: center;
  flex-shrink: 0;
}

.context-menu-separator {
  height: 1px;
  background: #555;
  margin: 0.1875rem 0; /* Reduced from 0.25rem 0 */
}

.file-details {
  display: flex;
  flex-direction: column;
  gap: 0.25rem; /* Reduced from 0.3rem */
  font-size: 0.625rem; /* Reduced from 0.7rem */
}

.detail-row {
  display: flex;
  justify-content: space-between;
  gap: 0.75rem; /* Reduced from 1rem */
}

.detail-label {
  color: #888;
  font-weight: 500;
  flex-shrink: 0;
}

.detail-value {
  color: #ccc;
  text-align: right;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 150px;
}

.btn-expand {
  background: #6c757d;
  transition: transform 0.2s;
}

.btn-expand:hover {
  background: #5a6268;
  transform: scale(1.1);
}

.path-breadcrumb {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  margin-bottom: 0.5rem;
  overflow-x: auto;
  white-space: nowrap;
  padding-bottom: 0.25rem;
}

.breadcrumb-item {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.breadcrumb-btn {
  background: #1e1e1e;
  border: 1px solid #555;
  color: #fff;
  padding: 0.2rem 0.4rem;
  border-radius: 3px;
  font-size: 0.7rem;
  cursor: pointer;
  white-space: nowrap;
}

.breadcrumb-btn:hover {
  background: #404040;
}

.breadcrumb-btn.active {
  background: #0d6efd;
  border-color: #0d6efd;
}

.breadcrumb-separator {
  color: #888;
  font-size: 0.8rem;
}

.path-controls {
  display: flex;
  gap: 0.5rem;
}

.path-input {
  flex: 1;
  background: #1e1e1e;
  border: 1px solid #555;
  color: #fff;
  padding: 0.25rem 0.5rem;
  border-radius: 3px;
  font-size: 0.75rem;
}

.path-input:focus {
  outline: none;
  border-color: #0d6efd;
}

.file-list-container {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: 1rem;
  color: #888;
}

.empty-icon {
  font-size: 2rem;
}

.empty-text {
  font-size: 0.8125rem; /* Reduced from 0.875rem */
}

.file-list {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.file-list-header {
  display: grid;
  grid-template-columns: 1fr 80px 100px 80px;
  gap: 0.375rem; /* Reduced from 0.5rem */
  padding: 0.375rem 0.5rem; /* Reduced from 0.5rem 0.75rem */
  background: #404040;
  border-bottom: 1px solid #555;
  font-size: 0.625rem; /* Reduced from 0.7rem */
  font-weight: 600;
  color: #ccc;
  flex-shrink: 0;
}

.file-list-body {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  min-height: 0;
}

/* Custom scrollbar for file list */
.file-list-body::-webkit-scrollbar {
  width: 8px;
}

.file-list-body::-webkit-scrollbar-track {
  background: #1e1e1e;
}

.file-list-body::-webkit-scrollbar-thumb {
  background: #555;
  border-radius: 4px;
}

.file-list-body::-webkit-scrollbar-thumb:hover {
  background: #777;
}

.file-item {
  display: grid;
  grid-template-columns: 1fr 80px 100px 80px;
  gap: 0.375rem; /* Reduced from 0.5rem */
  padding: 0.375rem 0.5rem; /* Reduced from 0.5rem 0.75rem */
  border-bottom: 1px solid #404040;
  cursor: pointer;
  font-size: 0.6875rem; /* Reduced from 0.75rem */
  align-items: center;
  transition: background-color 0.2s;
}

.file-item:hover {
  background: #404040;
}

.file-item.is-directory {
  font-weight: 500;
}

.file-item.is-parent {
  background: #2a2a2a;
  font-weight: 600;
  color: #ffd700;
}

.file-info {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  overflow: hidden;
}

.file-icon {
  width: 16px;
  text-align: center;
  flex-shrink: 0;
}

.file-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-size {
  color: #888;
  font-size: 0.7rem;
  text-align: right;
}

.file-modified {
  color: #888;
  font-size: 0.7rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-actions {
  display: flex;
  gap: 0.25rem;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.2s;
}

.file-item:hover .file-actions {
  opacity: 1;
}

.sftp-status {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 0.75rem;
  background: #353535;
  border-top: 1px solid #555;
  font-size: 0.7rem;
  flex-shrink: 0;
}

.current-path {
  color: #888;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 200px;
}

.connection-status {
  color: #888;
  white-space: nowrap;
}

.btn {
  background: #0d6efd;
  color: white;
  border: none;
  padding: 0.25rem 0.5rem;
  border-radius: 3px;
  font-size: 0.75rem;
  cursor: pointer;
  transition: background 0.2s;
}

.btn:hover {
  background: #0b5ed7;
}

.btn-sm {
  padding: 0.375rem 0.75rem;
  font-size: 0.875rem;
}

.btn-xs {
  padding: 0.125rem 0.25rem;
  font-size: 0.6rem;
}

.btn-danger {
  background: #dc3545;
}

.btn-danger:hover {
  background: #c82333;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-refresh {
  background: #28a745;
}

.btn-refresh:hover {
  background: #218838;
}

.btn-close {
  background: #dc3545;
  font-size: 1rem;
  line-height: 1;
}

.btn-up {
  background: #6c757d;
}

.btn-up:hover {
  background: #5a6268;
}

.btn-download {
  background: #17a2b8;
}

.btn-download:hover {
  background: #138496;
}

.form-control {
  background: #1e1e1e;
  border: 1px solid #555;
  color: #fff;
  padding: 0.375rem 0.75rem;
  border-radius: 3px;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

/* Custom Scrollbars for Dark Theme */
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

/* File browser specific scrollbar */
.file-browser :deep(*::-webkit-scrollbar-thumb) {
  background: #404040;
}

.file-browser :deep(*::-webkit-scrollbar-thumb:hover) {
  background: #505050;
}

/* Terminal output scrollbar */
.terminal-container :deep(*::-webkit-scrollbar-thumb) {
  background: #404040;
}

.terminal-container :deep(*::-webkit-scrollbar-thumb:hover) {
  background: #505050;
}
</style>
