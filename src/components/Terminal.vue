<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

interface Props {
  sessionId: string
  title?: string
}

const props = defineProps<Props>()

const terminalRef = ref<HTMLDivElement>()
const isConnected = ref(false)
const logs = ref<string[]>([
  `Welcome to TermNest Terminal`,
  `Session ID: ${props.sessionId}`,
  `Type 'help' for available commands`,
  ``
])

const currentInput = ref('')
const isInputFocused = ref(false)

// Simulate terminal commands
const commands: Record<string, () => string[]> = {
  help: () => [
    'Available commands:',
    '  help     - Show this help message',
    '  clear    - Clear the terminal',
    '  date     - Show current date',
    '  whoami   - Show current user',
    '  pwd      - Show current directory',
    '  ls       - List directory contents',
    '  exit     - Close the session',
    ''
  ],
  clear: () => {
    logs.value = []
    return []
  },
  date: () => [new Date().toString(), ''],
  whoami: () => ['user@' + (props.title || 'remote-host'), ''],
  pwd: () => ['/home/user', ''],
  ls: () => [
    'total 8',
    'drwxr-xr-x  2 user user 4096 Jan 29 17:30 .',
    'drwxr-xr-x  3 user user 4096 Jan 29 17:29 ..',
    '-rw-r--r--  1 user user  220 Jan 29 17:29 .bash_logout',
    '-rw-r--r--  1 user user 3771 Jan 29 17:29 .bashrc',
    '-rw-r--r--  1 user user  807 Jan 29 17:29 .profile',
    ''
  ],
  exit: () => {
    isConnected.value = false
    return ['Connection closed.', '']
  }
}

function executeCommand(command: string) {
  const cmd = command.trim().toLowerCase()
  logs.value.push(`$ ${command}`)
  
  if (commands[cmd]) {
    const output = commands[cmd]()
    logs.value.push(...output)
  } else if (cmd === '') {
    logs.value.push('')
  } else {
    logs.value.push(`Command not found: ${command}`, '')
  }
  
  scrollToBottom()
}

function handleInputSubmit() {
  if (currentInput.value.trim()) {
    executeCommand(currentInput.value)
    currentInput.value = ''
  }
}

function handleInputKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter') {
    event.preventDefault()
    handleInputSubmit()
  }
}

function focusInput() {
  const input = document.querySelector('.terminal-input') as HTMLInputElement
  if (input) {
    input.focus()
  }
}

function scrollToBottom() {
  setTimeout(() => {
    if (terminalRef.value) {
      terminalRef.value.scrollTop = terminalRef.value.scrollHeight
    }
  }, 10)
}

onMounted(() => {
  isConnected.value = true
  focusInput()
})

onUnmounted(() => {
  isConnected.value = false
})
</script>

<template>
  <div class="terminal-container">
    <div class="terminal-header">
      <div class="terminal-title">
        <span class="terminal-icon">🖥️</span>
        {{ title || `Session ${sessionId}` }}
      </div>
      <div class="terminal-controls">
        <div class="traffic-lights">
          <div class="traffic-light close" @click="() => {}"></div>
          <div class="traffic-light minimize" @click="() => {}"></div>
          <div class="traffic-light maximize" @click="() => {}"></div>
        </div>
        <div class="connection-status" :class="{ connected: isConnected }">
          <span class="status-dot"></span>
          {{ isConnected ? 'Connected' : 'Disconnected' }}
        </div>
      </div>
    </div>

    <div 
      ref="terminalRef"
      class="terminal-content"
      @click="focusInput"
    >
      <div class="terminal-output">
        <div 
          v-for="(line, index) in logs" 
          :key="index" 
          class="terminal-line"
          v-html="line"
        ></div>
      </div>
      
      <div v-if="isConnected" class="terminal-input-line">
        <span class="terminal-prompt">$ </span>
        <input
          v-model="currentInput"
          @keydown="handleInputKeydown"
          @focus="isInputFocused = true"
          @blur="isInputFocused = false"
          class="terminal-input"
          type="text"
          autocomplete="off"
          spellcheck="false"
        />
        <span class="terminal-cursor" :class="{ blinking: !isInputFocused }">_</span>
      </div>
      
      <div v-else class="terminal-disconnected">
        <span class="disconnected-message">Terminal session ended.</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.terminal-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #1a1a1a;
  border-radius: 0.5rem;
  overflow: hidden;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', 'Consolas', monospace;
  box-shadow: var(--shadow);
}

.terminal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: #2d2d2d;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid #404040;
}

.terminal-title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: #ffffff;
  font-weight: 500;
  font-size: 0.875rem;
}

.terminal-icon {
  font-size: 1rem;
}

.terminal-controls {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.traffic-lights {
  display: flex;
  gap: 0.5rem;
}

.traffic-light {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  cursor: pointer;
  transition: opacity 0.2s ease;
}

.traffic-light:hover {
  opacity: 0.8;
}

.traffic-light.close {
  background: #ff5f56;
}

.traffic-light.minimize {
  background: #ffbd2e;
}

.traffic-light.maximize {
  background: #27ca3f;
}

.connection-status {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.75rem;
  color: #b0b0b0;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #6c757d;
}

.connection-status.connected .status-dot {
  background: #28a745;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.terminal-content {
  flex: 1;
  background: #1a1a1a;
  color: #00ff00;
  padding: 1rem;
  overflow-y: auto;
  cursor: text;
  line-height: 1.4;
  font-size: 14px;
}

.terminal-output {
  margin-bottom: 0.5rem;
}

.terminal-line {
  margin-bottom: 0.25rem;
  word-wrap: break-word;
  white-space: pre-wrap;
}

.terminal-input-line {
  display: flex;
  align-items: center;
  color: #00ff00;
}

.terminal-prompt {
  color: #00ff00;
  margin-right: 0.5rem;
  user-select: none;
}

.terminal-input {
  background: transparent;
  border: none;
  color: #00ff00;
  font-family: inherit;
  font-size: inherit;
  outline: none;
  flex: 1;
  caret-color: transparent;
}

.terminal-cursor {
  color: #00ff00;
  animation: blink 1s infinite;
}

.terminal-cursor.blinking {
  animation: blink 1s infinite;
}

@keyframes blink {
  0%, 50% { opacity: 1; }
  51%, 100% { opacity: 0; }
}

.terminal-disconnected {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  color: #6c757d;
  font-style: italic;
}

.disconnected-message {
  background: #2d2d2d;
  padding: 1rem 2rem;
  border-radius: 0.375rem;
  border: 1px solid #404040;
}

/* Scrollbar styling for WebKit browsers */
.terminal-content::-webkit-scrollbar {
  width: 8px;
}

.terminal-content::-webkit-scrollbar-track {
  background: #2d2d2d;
}

.terminal-content::-webkit-scrollbar-thumb {
  background: #404040;
  border-radius: 4px;
}

.terminal-content::-webkit-scrollbar-thumb:hover {
  background: #555;
}
</style>
