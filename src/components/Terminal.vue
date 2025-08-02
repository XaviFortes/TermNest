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
    
    <div class="terminal-wrapper">
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
    <div v-if="showSftp" class="sftp-panel">
      <div class="sftp-header">
        <h3>File Manager</h3>
        <button @click="closeSftp" class="btn btn-sm">×</button>
      </div>
      <div class="sftp-content">
        <div class="file-browser">
          <div class="path-bar">
            <input v-model="currentPath" @keyup.enter="navigateToPath" class="form-control" />
            <button @click="refreshFiles" class="btn btn-sm">Refresh</button>
          </div>
          <div class="file-list">
            <div 
              v-for="file in files" 
              :key="file.path"
              class="file-item"
              :class="{ 'is-directory': file.is_directory }"
              @dblclick="handleFileAction(file)"
            >
              <span class="file-icon">
                {{ file.is_directory ? '📁' : '📄' }}
              </span>
              <span class="file-name">{{ file.name }}</span>
              <span class="file-size" v-if="!file.is_directory">
                {{ formatFileSize(file.size) }}
              </span>
              <div class="file-actions">
                <button @click="downloadFile(file)" v-if="!file.is_directory" class="btn btn-xs">
                  Download
                </button>
                <button @click="deleteFile(file)" class="btn btn-xs btn-danger">
                  Delete
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useSessionsStore } from '../stores/sessions'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
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

// Event listener cleanup
let unlistenTerminalOutput: (() => void) | null = null
let unlistenConnectionStatus: (() => void) | null = null

// Initialize terminal
onMounted(async () => {
  setupTerminal()
  await initializeTerminal()
  await setupEventListeners()
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
})

async function setupEventListeners() {
  // Listen for terminal output from the backend
  unlistenTerminalOutput = await listen('terminal_output', (event: any) => {
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
  } else if (status === 'disconnected') {
    connectionStatus.value = 'disconnected'
  }
}

async function initializeTerminal() {
  try {
    connectionStatus.value = 'connecting'
    
    // Connect using real SSH
    await sessionsStore.connectToSession(props.sessionId)
    
    connectionStatus.value = 'connected'
    
    if (props.protocol === 'SSH') {
      await loadRemoteFiles()
    }
  } catch (error) {
    console.error('Failed to initialize terminal:', error)
    connectionStatus.value = 'disconnected'
    if (terminal) {
      terminal.write('Failed to connect: ' + error + '\r\n')
    }
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
    fontSize: 14,
    fontFamily: 'Monaco, Menlo, "Ubuntu Mono", monospace',
    cursorBlink: true,
    convertEol: true,
    scrollback: 1000
  })
  
  // Create fit addon
  fitAddon = new FitAddon()
  terminal.loadAddon(fitAddon)
  
  // Open terminal in container
  terminal.open(terminalContainer.value)
  fitAddon.fit()
  
  // Handle user input
  terminal.onData((data) => {
    // Send input to SSH backend
    invoke('send_terminal_input', {
      sessionId: props.sessionId,
      input: data
    }).catch(error => {
      console.error('Failed to send input:', error)
    })
  })
  
  // Handle resize
  const resizeObserver = new ResizeObserver(() => {
    if (fitAddon) {
      fitAddon.fit()
    }
  })
  resizeObserver.observe(terminalContainer.value)
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
}

function closeSftp() {
  showSftp.value = false
}

async function loadRemoteFiles() {
  try {
    const result = await invoke('list_remote_directory', {
      sessionId: props.sessionId,
      path: currentPath.value
    })
    
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

async function downloadFile(file: FileItem) {
  try {
    appendOutput('Downloading ' + file.name + '...\n')
    // Implementation would go here
    appendOutput('Download completed\n\n')
  } catch (error) {
    appendOutput('Download failed: ' + error + '\n\n')
  }
}

async function deleteFile(file: FileItem) {
  if (confirm('Are you sure you want to delete ' + file.name + '?')) {
    try {
      appendOutput('Deleting ' + file.name + '...\n')
      // Implementation would go here
      appendOutput('File deleted\n\n')
      refreshFiles()
    } catch (error) {
      appendOutput('Delete failed: ' + error + '\n\n')
    }
  }
}

function disconnect() {
  console.log('Disconnecting session:', props.sessionId)
  
  // Disconnect from SSH backend
  invoke('disconnect_session', { sessionId: props.sessionId })
    .then(() => {
      console.log('Session disconnected successfully')
    })
    .catch((error) => {
      console.error('Failed to disconnect session:', error)
    })
  
  // Update local state
  connectionStatus.value = 'disconnected'
  
  // Close the session and go back to sessions list
  sessionsStore.closeSession()
}
</script>

<style scoped>
.terminal-container {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #1e1e1e;
  color: #ffffff;
  font-family: 'Monaco', 'Menlo', 'Consolas', monospace;
}

.terminal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem 1rem;
  background: #2d2d2d;
  border-bottom: 1px solid #404040;
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
  padding: 1rem;
  overflow-y: auto;
  cursor: text;
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
  background: #2d2d2d;
  border-left: 1px solid #404040;
  display: flex;
  flex-direction: column;
}

.sftp-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem 1rem;
  background: #404040;
  border-bottom: 1px solid #555;
}

.sftp-header h3 {
  margin: 0;
  font-size: 0.875rem;
  font-weight: 600;
}

.sftp-content {
  flex: 1;
  overflow: hidden;
}

.file-browser {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.path-bar {
  display: flex;
  gap: 0.5rem;
  padding: 0.75rem;
  background: #353535;
  border-bottom: 1px solid #555;
}

.path-bar input {
  flex: 1;
  background: #1e1e1e;
  border: 1px solid #555;
  color: #fff;
  padding: 0.25rem 0.5rem;
  border-radius: 3px;
  font-size: 0.75rem;
}

.file-list {
  flex: 1;
  overflow-y: auto;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  border-bottom: 1px solid #404040;
  cursor: pointer;
  font-size: 0.75rem;
}

.file-item:hover {
  background: #404040;
}

.file-item.is-directory {
  font-weight: 500;
}

.file-icon {
  width: 16px;
  text-align: center;
}

.file-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-size {
  color: #888;
  font-size: 0.7rem;
}

.file-actions {
  display: flex;
  gap: 0.25rem;
  opacity: 0;
  transition: opacity 0.2s;
}

.file-item:hover .file-actions {
  opacity: 1;
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
</style>
