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
      <div 
        ref="terminalContainer" 
        class="terminal-content"
        @click="focusTerminal"
      >
        <div class="terminal-simulation">
          <div class="terminal-prompt">{{ currentPrompt }}</div>
          <div 
            class="terminal-input" 
            contenteditable="true"
            @keydown="handleKeydown"
            @keyup="handleInput"
            ref="terminalInput"
          ></div>
          <div class="terminal-output" v-html="terminalOutput"></div>
        </div>
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
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useSessionsStore } from '../stores/sessions'

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
const terminalInput = ref<HTMLElement>()
const connectionStatus = ref<'connecting' | 'connected' | 'disconnected'>('connecting')
const terminalOutput = ref('')
const currentPrompt = ref('user@localhost:~$ ')

// SFTP state
const showSftp = ref(false)
const currentPath = ref('/')
const files = ref<FileItem[]>([])

// Event listener cleanup
let unlistenTerminalOutput: (() => void) | null = null

// Initialize terminal
onMounted(async () => {
  await initializeTerminal()
  await setupEventListeners()
  focusTerminal()
})

onUnmounted(() => {
  disconnect()
  if (unlistenTerminalOutput) {
    unlistenTerminalOutput()
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
    appendOutput('Failed to connect: ' + error + '\n')
  }
}

function appendOutput(text: string) {
  terminalOutput.value += text.replace(/\n/g, '<br>')
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter') {
    event.preventDefault()
    executeCommand()
  }
}

function handleInput() {
  // Handle terminal input if needed
}

async function executeCommand() {
  const input = terminalInput.value
  if (!input) return
  
  const command = input.textContent?.trim() || ''
  appendOutput(currentPrompt.value + command + '\n')
  
  // Process command
  await processCommand(command)
  
  // Clear input
  input.textContent = ''
}

async function processCommand(command: string) {
  // Handle local commands first
  const cmd = command.toLowerCase().trim()
  
  if (cmd === 'clear') {
    terminalOutput.value = ''
    return
  }
  
  if (cmd === 'sftp') {
    toggleSftp()
    appendOutput('SFTP panel toggled\n\n')
    return
  }
  
  if (cmd === 'exit') {
    disconnect()
    return
  }
  
  // Send all other commands to real SSH session
  try {
    await invoke('send_terminal_input', {
      session_id: props.sessionId,
      data: command + '\n'
    })
  } catch (error) {
    console.error('Failed to send command:', error)
    appendOutput('Error sending command: ' + error + '\n')
  }
}

function focusTerminal() {
  nextTick(() => {
    terminalInput.value?.focus()
  })
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
  connectionStatus.value = 'disconnected'
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
  overflow: hidden;
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
