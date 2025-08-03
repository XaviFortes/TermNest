<template>
  <div class="password-dialog-overlay" @click="handleOverlayClick">
    <div class="password-dialog" @click.stop>
      <div class="password-dialog-header">
        <h3>SSH Authentication Required</h3>
        <button @click="cancel" class="close-button" type="button">×</button>
      </div>
      
      <div class="password-dialog-body">
        <div class="connection-info">
          <p><strong>Host:</strong> {{ host }}</p>
          <p><strong>Username:</strong> {{ username }}</p>
        </div>
        
        <div class="form-group">
          <label for="password">Password:</label>
          <input
            id="password"
            ref="passwordInput"
            v-model="password"
            type="password"
            class="form-control"
            placeholder="Enter your password"
            @keyup.enter="authenticate"
            @keyup.escape="cancel"
            autocomplete="current-password"
          />
        </div>
        
        <div v-if="error" class="error-message">
          {{ error }}
        </div>
      </div>
      
      <div class="password-dialog-footer">
        <button @click="cancel" class="btn btn-secondary" type="button">
          Cancel
        </button>
        <button 
          @click="authenticate" 
          class="btn btn-primary" 
          type="button"
          :disabled="!password || isAuthenticating"
        >
          <span v-if="isAuthenticating" class="spinner"></span>
          {{ isAuthenticating ? 'Connecting...' : 'Connect' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, onMounted } from 'vue'

interface Props {
  host: string
  username: string
  error?: string
  isAuthenticating?: boolean
}

interface Emits {
  (e: 'authenticate', password: string): void
  (e: 'cancel'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const password = ref('')
const passwordInput = ref<HTMLInputElement>()

const authenticate = () => {
  if (password.value && !props.isAuthenticating) {
    emit('authenticate', password.value)
  }
}

const cancel = () => {
  emit('cancel')
}

const handleOverlayClick = () => {
  cancel()
}

onMounted(async () => {
  // Focus the password input when the dialog opens
  await nextTick()
  passwordInput.value?.focus()
})
</script>

<style scoped>
.password-dialog-overlay {
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
}

.password-dialog {
  /* background: var(--background-color); */
  background: rgba(0, 0, 0, 0.9);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  min-width: 400px;
  max-width: 500px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.password-dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
  border-bottom: 1px solid var(--border-color);
}

.password-dialog-header h3 {
  margin: 0;
  color: var(--text-color);
}

.close-button {
  background: none;
  border: none;
  font-size: 1.5rem;
  cursor: pointer;
  color: var(--text-color);
  padding: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-button:hover {
  background: var(--hover-color);
  border-radius: 4px;
}

.password-dialog-body {
  padding: 1.5rem;
}

.connection-info {
  margin-bottom: 1rem;
  padding: 1rem;
  background: var(--secondary-background);
  border-radius: 4px;
  border-left: 3px solid var(--primary-color);
}

.connection-info p {
  margin: 0.25rem 0;
  color: var(--text-color);
}

.form-group {
  margin-bottom: 1rem;
}

.form-group label {
  display: block;
  margin-bottom: 0.5rem;
  color: var(--text-color);
  font-weight: 500;
}

.form-control {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--input-background);
  color: var(--text-color);
  font-size: 1rem;
}

.form-control:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
}

.error-message {
  color: var(--error-color);
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: 4px;
  padding: 0.75rem;
  font-size: 0.875rem;
}

.password-dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.75rem;
  padding: 1rem;
  border-top: 1px solid var(--border-color);
}

.btn {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.875rem;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  transition: all 0.2s;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-secondary {
  background: var(--secondary-background);
  color: var(--text-color);
  border: 1px solid var(--border-color);
}

.btn-secondary:hover:not(:disabled) {
  background: var(--hover-color);
}

.btn-primary {
  background: var(--primary-color);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: var(--primary-color-hover);
}

.spinner {
  width: 12px;
  height: 12px;
  border: 2px solid transparent;
  border-top: 2px solid currentColor;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* CSS Variables for theming */
:root {
  --background-color: #ffffff;
  --secondary-background: #f8fafc;
  --text-color: #1f2937;
  --border-color: #d1d5db;
  --input-background: #ffffff;
  --primary-color: #3b82f6;
  --primary-color-hover: #2563eb;
  --hover-color: #f3f4f6;
  --error-color: #ef4444;
}

[data-theme="dark"] {
  --background-color: #1f2937;
  --secondary-background: #374151;
  --text-color: #f9fafb;
  --border-color: #4b5563;
  --input-background: #374151;
  --primary-color: #3b82f6;
  --primary-color-hover: #2563eb;
  --hover-color: #4b5563;
  --error-color: #ef4444;
}
</style>
