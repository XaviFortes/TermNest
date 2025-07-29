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
      ></div>
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
              <span class="file-modified">{{ file.modified }}</span>
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

interface Props {
  sessionId: string
  sessionName: string
  protocol?: string
}

const props = withDefaults(defineProps<Props>(), {
  protocol: 'SSH'
})

const terminalContainer = ref<HTMLElement>()
const connectionStatus = ref<'disconnected' | 'connecting' | 'connected' | 'error'>('disconnected')
const showSftp = ref(false)
const currentPath = ref('/home')
const files = ref<any[]>([])

// Terminal state
let terminal: any = null

onMounted(async () => {
  await initializeTerminal()
  await connectToSession()
})

onUnmounted(() => {
  if (terminal) {
    terminal.dispose()
  }
})

async function initializeTerminal() {
  // For now, we'll use a simple terminal simulation
  // In a real implementation, you'd use xterm.js
  
  if (!terminalContainer.value) return
  
  const terminalElement = document.createElement('div')
  terminalElement.className = 'xterm-simulation'
  terminalElement.contentEditable = 'true'
  terminalElement.style.cssText = `
    background: #000;
    color: #fff;
    font-family: 'Courier New', monospace;
    font-size: 14px;
    padding: 10px;
    height: 400px;
    overflow-y: auto;
    white-space: pre-wrap;
    outline: none;
  `
  
  terminalElement.innerHTML = 'TermNest Terminal\n$ '
  
  terminalContainer.value.appendChild(terminalElement)
  
  // Add basic keyboard handling
  terminalElement.addEventListener('keydown', handleKeydown)
}

function handleKeydown(e: KeyboardEvent) {
  const terminalElement = e.target as HTMLElement
  
  if (e.key === 'Enter') {
    e.preventDefault()
    const content = terminalElement.textContent || ''
    const command = getCurrentCommand(content)
    
    if (command.trim()) {
      sendCommand(command)
    }
    
    // Add new prompt line
    terminalElement.textContent += '\n$ '
    
    // Move cursor to end
    const range = document.createRange()
    const selection = window.getSelection()
    range.selectNodeContents(terminalElement)
    range.collapse(false)
    selection?.removeAllRanges()
    selection?.addRange(range)
  }
}

function getCurrentCommand(content: string): string {
  const lines = content.split('\n')
  const lastLine = lines[lines.length - 1]
  return lastLine.replace(/^.*\$ /, '')
}

async function sendCommand(command: string) {
  try {
    console.log('Sending command:', command)
    await invoke('send_terminal_input', {
      input: {
        session_id: props.sessionId,
        data: command + '\n'
      }
    })
  } catch (error) {
    console.error('Failed to send command:', error)
    // Simulate response for demo
    simulateCommandResponse(command)
  }
}

function simulateCommandResponse(command: string) {
  const terminalElement = terminalContainer.value?.querySelector('.xterm-simulation')
  if (!terminalElement) return
  
  let response = ''
  
  switch (command.trim()) {
    case 'ls':
      response = 'file1.txt  file2.txt  directory1/  directory2/'
      break
    case 'pwd':
      response = '/home/user'
      break
    case 'whoami':
      response = 'user'
      break
    case 'date':
      response = new Date().toString()
      break
    case 'clear':
      terminalElement.textContent = '$ '
      return
    default:
      if (command.trim()) {
        response = `Command '${command}' not found`
      }
  }
  
  if (response) {
    terminalElement.textContent += '\n' + response
  }
}

async function connectToSession() {
  try {
    connectionStatus.value = 'connecting'
    console.log('Connecting to session:', props.sessionId)
    await invoke('connect_ssh', { sessionId: props.sessionId })
    connectionStatus.value = 'connected'
  } catch (error) {
    console.error('Failed to connect:', error)
    connectionStatus.value = 'error'
  }
}

async function disconnect() {
  try {
    await invoke('disconnect_session', { sessionId: props.sessionId })
    connectionStatus.value = 'disconnected'
  } catch (error) {
    console.error('Failed to disconnect:', error)
  }
}

function focusTerminal() {
  const terminalElement = terminalContainer.value?.querySelector('.xterm-simulation')
  if (terminalElement) {
    (terminalElement as HTMLElement).focus()
  }
}

async function toggleSftp() {
  if (!showSftp.value) {
    try {
      await invoke('create_sftp_session', { sessionId: props.sessionId })
      await refreshFiles()
      showSftp.value = true
    } catch (error) {
      console.error('Failed to create SFTP session:', error)
      // Simulate for demo
      files.value = [
        { name: '..', path: '/home', is_directory: true, size: 0, modified: '2024-01-29 12:00' },
        { name: 'documents', path: '/home/user/documents', is_directory: true, size: 0, modified: '2024-01-29 12:00' },
        { name: 'file1.txt', path: '/home/user/file1.txt', is_directory: false, size: 1024, modified: '2024-01-29 12:00' },
        { name: 'file2.txt', path: '/home/user/file2.txt', is_directory: false, size: 2048, modified: '2024-01-29 12:00' },
      ]
      showSftp.value = true
    }
  } else {
    showSftp.value = false
  }
}

function closeSftp() {
  showSftp.value = false
}

async function refreshFiles() {
  try {
    const fileList = await invoke('list_remote_directory', {
      sessionId: props.sessionId,
      path: currentPath.value
    })
    files.value = fileList as any[]
  } catch (error) {
    console.error('Failed to list files:', error)
  }
}

async function navigateToPath() {
  await refreshFiles()
}

function handleFileAction(file: any) {
  if (file.is_directory) {
    currentPath.value = file.path
    refreshFiles()
  }
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}
</script>

<style scoped>
.terminal-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #1e1e1e;
  border-radius: 8px;
  overflow: hidden;
}

.terminal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 16px;
  background: #2d2d2d;
  border-bottom: 1px solid #404040;
}

.terminal-title {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #fff;
  font-weight: 500;
}

.connection-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #666;
}

.connection-indicator.connected {
  background: #4ade80;
}

.connection-indicator.connecting {
  background: #fbbf24;
  animation: pulse 1s infinite;
}

.connection-indicator.error {
  background: #ef4444;
}

.terminal-actions {
  display: flex;
  gap: 8px;
}

.terminal-wrapper {
  flex: 1;
  overflow: hidden;
}

.terminal-content {
  height: 100%;
  background: #000;
}

.xterm-simulation {
  cursor: text;
  outline: none;
}

.xterm-simulation:focus {
  outline: none;
}

.sftp-panel {
  height: 300px;
  border-top: 1px solid #404040;
  background: #2d2d2d;
  display: flex;
  flex-direction: column;
}

.sftp-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 16px;
  background: #3d3d3d;
  color: #fff;
}

.sftp-content {
  flex: 1;
  padding: 16px;
  overflow: hidden;
}

.path-bar {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
}

.path-bar input {
  flex: 1;
}

.file-list {
  height: 200px;
  overflow-y: auto;
  border: 1px solid #404040;
  border-radius: 4px;
}

.file-item {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  border-bottom: 1px solid #404040;
  color: #fff;
  cursor: pointer;
  transition: background-color 0.2s;
}

.file-item:hover {
  background: #404040;
}

.file-item.is-directory {
  font-weight: 500;
}

.file-icon {
  margin-right: 8px;
  font-size: 16px;
}

.file-name {
  flex: 1;
  margin-right: 16px;
}

.file-size,
.file-modified {
  font-size: 12px;
  color: #999;
  margin-left: 16px;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}
</style>
