<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useSessionsStore, type Protocol, type Session } from '../stores/sessions'
import { invoke } from '@tauri-apps/api/core'

interface Props {
  editingSession?: Session
}

interface Emits {
  (e: 'close'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()
const sessionsStore = useSessionsStore()

const formData = reactive({
  name: '',
  host: '',
  port: 22,
  username: '',
  protocol: 'SSH' as Protocol,
  authMethod: 'PublicKey' as 'Password' | 'PublicKey' | 'Agent',
  keyPath: '',
  password: ''
})

const isSubmitting = ref(false)
const errors = ref<Record<string, string>>({})
const isEditing = ref(false)

// Set default SSH key path based on platform
const getDefaultKeyPath = () => {
  // Check if we're on Windows by looking at the navigator platform
  const isWindows = navigator.platform.toLowerCase().includes('win')
  return isWindows ? '%USERPROFILE%\\.ssh\\id_ed25519' : '~/.ssh/id_ed25519'
}

// Initialize form data when editing
onMounted(() => {
  if (props.editingSession) {
    isEditing.value = true
    formData.name = props.editingSession.name
    formData.host = props.editingSession.host
    formData.port = props.editingSession.port
    formData.username = props.editingSession.username
    formData.protocol = props.editingSession.protocol
    
    // Handle auth method
    if (props.editingSession.auth_method === 'Password') {
      formData.authMethod = 'Password'
    } else if (props.editingSession.auth_method === 'Agent') {
      formData.authMethod = 'Agent'
    } else if (typeof props.editingSession.auth_method === 'object' && 'PublicKey' in props.editingSession.auth_method) {
      formData.authMethod = 'PublicKey'
      formData.keyPath = props.editingSession.auth_method.PublicKey.key_path
    }
  } else {
    // Set default key path for new sessions
    formData.keyPath = getDefaultKeyPath()
  }
})

async function browseSSHKey() {
  try {
    const result = await invoke<string | null>('browse_ssh_key')
    if (result) {
      formData.keyPath = result
    }
  } catch (error) {
    console.error('Failed to browse SSH key:', error)
  }
}

function validateForm() {
  errors.value = {}
  
  if (!formData.name.trim()) {
    errors.value.name = 'Session name is required'
  }
  
  if (!formData.host.trim()) {
    errors.value.host = 'Host is required'
  }
  
  if (!formData.username.trim()) {
    errors.value.username = 'Username is required'
  }
  
  if (formData.port < 1 || formData.port > 65535) {
    errors.value.port = 'Port must be between 1 and 65535'
  }
  
  if (formData.authMethod === 'PublicKey' && !formData.keyPath.trim()) {
    errors.value.keyPath = 'SSH key path is required'
  }
  
  return Object.keys(errors.value).length === 0
}

async function handleSubmit() {
  if (!validateForm()) {
    return
  }
  
  isSubmitting.value = true
  
  try {
    const sessionData = {
      name: formData.name,
      host: formData.host,
      port: formData.port,
      username: formData.username,
      protocol: formData.protocol,
      auth_method: formData.authMethod === 'Password' 
        ? 'Password' as const
        : formData.authMethod === 'Agent'
        ? 'Agent' as const
        : { PublicKey: { key_path: formData.keyPath } }
    }
    
    if (isEditing.value && props.editingSession) {
      // Update existing session
      console.log('Updating session with data:', sessionData)
      const updatedSession = {
        ...props.editingSession,
        ...sessionData,
        last_used: props.editingSession.last_used // Preserve last_used timestamp
      }
      
      const timeoutPromise = new Promise((_, reject) => {
        setTimeout(() => reject(new Error('Session update timed out')), 10000)
      })
      
      await Promise.race([
        sessionsStore.updateSession(updatedSession),
        timeoutPromise
      ])
      
      console.log('Session updated successfully')
    } else {
      // Create new session
      console.log('Creating session with data:', sessionData)
      
      const timeoutPromise = new Promise((_, reject) => {
        setTimeout(() => reject(new Error('Session creation timed out')), 10000)
      })
      
      await Promise.race([
        sessionsStore.createSession(sessionData),
        timeoutPromise
      ])
      
      console.log('Session created successfully')
    }
    
    emit('close')
  } catch (error) {
    console.error('Failed to save session:', error)
    // Show the error to the user
    errors.value.general = error instanceof Error ? error.message : String(error)
  } finally {
    isSubmitting.value = false
  }
}

function handleCancel() {
  emit('close')
}

function handleOverlayClick(event: MouseEvent) {
  if (event.target === event.currentTarget) {
    emit('close')
  }
}
</script>

<template>
  <div class="modal-overlay" @click="handleOverlayClick">
    <div class="modal-container">
      <div class="modal-header">
        <h2 class="modal-title">{{ isEditing ? 'Edit Session' : 'Create New Session' }}</h2>
        <button class="modal-close" @click="handleCancel">
          ✕
        </button>
      </div>

      <form @submit.prevent="handleSubmit" class="modal-form">
        <!-- General Error Display -->
        <div v-if="errors.general" class="form-error general-error">
          <span class="error-icon">⚠️</span>
          {{ errors.general }}
        </div>

        <div class="form-group">
          <label for="name" class="form-label">Session Name</label>
          <input
            id="name"
            v-model="formData.name"
            type="text"
            class="form-input"
            :class="{ error: errors.name }"
            placeholder="e.g. Production Server"
            autocomplete="off"
          />
          <div v-if="errors.name" class="form-error">{{ errors.name }}</div>
        </div>

        <div class="form-row">
          <div class="form-group">
            <label for="host" class="form-label">Host</label>
            <input
              id="host"
              v-model="formData.host"
              type="text"
              class="form-input"
              :class="{ error: errors.host }"
              placeholder="example.com or 192.168.1.100"
              autocomplete="off"
            />
            <div v-if="errors.host" class="form-error">{{ errors.host }}</div>
          </div>

          <div class="form-group">
            <label for="port" class="form-label">Port</label>
            <input
              id="port"
              v-model.number="formData.port"
              type="number"
              min="1"
              max="65535"
              class="form-input"
              :class="{ error: errors.port }"
            />
            <div v-if="errors.port" class="form-error">{{ errors.port }}</div>
          </div>
        </div>

        <div class="form-group">
          <label for="username" class="form-label">Username</label>
          <input
            id="username"
            v-model="formData.username"
            type="text"
            class="form-input"
            :class="{ error: errors.username }"
            placeholder="ubuntu, root, etc."
            autocomplete="username"
          />
          <div v-if="errors.username" class="form-error">{{ errors.username }}</div>
        </div>

        <div class="form-group">
          <label for="protocol" class="form-label">Protocol</label>
          <select
            id="protocol"
            v-model="formData.protocol"
            class="form-select"
          >
            <option value="SSH">SSH</option>
            <option value="SFTP">SFTP</option>
            <option value="RDP" disabled>RDP (Coming Soon)</option>
            <option value="VNC" disabled>VNC (Coming Soon)</option>
            <option value="Telnet" disabled>Telnet (Coming Soon)</option>
          </select>
        </div>

        <div class="form-group">
          <label class="form-label">Authentication Method</label>
          <div class="radio-group">
            <label class="radio-option">
              <input
                type="radio"
                v-model="formData.authMethod"
                value="PublicKey"
                class="radio-input"
              />
              <span class="radio-label">SSH Key (Recommended)</span>
            </label>
            <label class="radio-option">
              <input
                type="radio"
                v-model="formData.authMethod"
                value="Agent"
                class="radio-input"
              />
              <span class="radio-label">SSH Agent</span>
            </label>
            <label class="radio-option">
              <input
                type="radio"
                v-model="formData.authMethod"
                value="Password"
                class="radio-input"
              />
              <span class="radio-label">Password</span>
            </label>
          </div>
        </div>

        <div v-if="formData.authMethod === 'PublicKey'" class="form-group">
          <label for="keyPath" class="form-label">SSH Key Path</label>
          <div class="input-with-button">
            <input
              id="keyPath"
              v-model="formData.keyPath"
              type="text"
              class="form-input"
              :class="{ error: errors.keyPath }"
              placeholder="Select or enter SSH key path..."
            />
            <button
              type="button"
              @click="browseSSHKey"
              class="btn btn-browse"
              title="Browse for SSH key file"
            >
              Browse
            </button>
          </div>
          <div v-if="errors.keyPath" class="form-error">{{ errors.keyPath }}</div>
          <div class="form-help">
            Path to your private SSH key file
          </div>
        </div>

        <div class="modal-actions">
          <button
            type="button"
            @click="handleCancel"
            class="btn btn-secondary"
            :disabled="isSubmitting"
          >
            Cancel
          </button>
          <button
            type="submit"
            class="btn btn-primary"
            :disabled="isSubmitting"
          >
            <span v-if="isSubmitting">{{ isEditing ? 'Updating...' : 'Creating...' }}</span>
            <span v-else>{{ isEditing ? 'Update Session' : 'Create Session' }}</span>
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 1rem;
}

.modal-container {
  background: var(--bg-primary);
  border-radius: 0.5rem;
  box-shadow: var(--shadow-hover);
  width: 100%;
  max-width: 500px;
  max-height: 90vh;
  overflow-y: auto;
  border: 1px solid var(--border-color);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.5rem 1.5rem 0;
  margin-bottom: 1.5rem;
}

.modal-title {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--text-primary);
}

.modal-close {
  background: none;
  border: none;
  font-size: 1.25rem;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0.25rem;
  border-radius: 0.25rem;
  transition: all 0.2s ease;
}

.modal-close:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.modal-form {
  padding: 0 1.5rem 1.5rem;
}

.form-group {
  margin-bottom: 1.5rem;
}

.form-row {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: 1rem;
}

.form-label {
  display: block;
  font-weight: 500;
  color: var(--text-primary);
  margin-bottom: 0.5rem;
}

.form-input,
.form-select {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.875rem;
  transition: border-color 0.2s ease;
}

.form-input:focus,
.form-select:focus {
  outline: none;
  border-color: var(--text-accent);
}

.form-input.error {
  border-color: #dc3545;
}

.form-error {
  color: #dc3545;
  font-size: 0.75rem;
  margin-top: 0.25rem;
}

.form-help {
  color: var(--text-secondary);
  font-size: 0.75rem;
  margin-top: 0.25rem;
}

.input-with-button {
  display: flex;
  gap: 0.5rem;
}

.input-with-button .form-input {
  flex: 1;
}

.btn-browse {
  padding: 0.75rem 1rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  color: var(--text-primary);
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.btn-browse:hover {
  background: var(--bg-accent);
  border-color: var(--text-accent);
}

.general-error {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  background: #fee;
  color: #c53030;
  padding: 1rem;
  border-radius: 0.375rem;
  border: 1px solid #fed7d7;
  margin-bottom: 1.5rem;
}

.error-icon {
  flex-shrink: 0;
}

.radio-group {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.radio-option {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
}

.radio-input {
  margin: 0;
}

.radio-label {
  color: var(--text-primary);
  font-size: 0.875rem;
}

.modal-actions {
  display: flex;
  gap: 1rem;
  justify-content: flex-end;
  margin-top: 2rem;
  padding-top: 1.5rem;
  border-top: 1px solid var(--border-color);
}

@media (max-width: 600px) {
  .modal-overlay {
    padding: 0.5rem;
  }
  
  .modal-container {
    max-height: 95vh;
  }
  
  .form-row {
    grid-template-columns: 1fr;
  }
  
  .modal-actions {
    flex-direction: column;
  }
}
</style>
