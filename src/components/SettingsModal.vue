<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSettingsStore } from '../stores/settings'
import { useThemesStore } from '../stores/themes'

interface Emits {
  (e: 'close'): void
}

const emit = defineEmits<Emits>()
const settingsStore = useSettingsStore()
const themesStore = useThemesStore()

const activeTab = ref('general')

const protocols = [
  { value: 'SSH', label: 'SSH' },
  { value: 'SFTP', label: 'SFTP' }
]

const terminalThemes = [
  { value: 'dark', label: 'Dark' },
  { value: 'light', label: 'Light' },
  { value: 'monokai', label: 'Monokai' },
  { value: 'solarized', label: 'Solarized' }
]

const isDirty = ref(false)
const localSettings = ref({ ...settingsStore.settings })

// Sync localSettings with current theme when modal opens
onMounted(() => {
  console.log('SettingsModal mounted')
  console.log('Current theme ID:', themesStore.currentThemeId)
  console.log('Settings store theme:', settingsStore.settings.theme)
  console.log('Available themes:', themesStore.availableThemes.map(t => `${t.metadata.id}: ${t.metadata.name}`))
  
  // Ensure localSettings reflects the current theme
  localSettings.value = { 
    ...settingsStore.settings,
    theme: themesStore.currentThemeId || settingsStore.settings.theme
  }
  
  console.log('LocalSettings theme set to:', localSettings.value.theme)
  
  // Debug available themes
  if (themesStore.debugThemes) {
    themesStore.debugThemes()
  }
})

function markDirty() {
  isDirty.value = true
}

async function handleThemeChange() {
  console.log('Theme change triggered, new value:', localSettings.value.theme)
  try {
    await themesStore.setTheme(localSettings.value.theme)
    // Also update the settings store immediately
    await settingsStore.updateSetting('theme', localSettings.value.theme)
    console.log('Theme applied successfully')
  } catch (error) {
    console.error('Failed to apply theme:', error)
  }
}

async function saveSettings() {
  try {
    // Apply the theme first if it changed
    if (localSettings.value.theme !== settingsStore.settings.theme) {
      console.log('Theme changed during save, applying:', localSettings.value.theme)
      await themesStore.setTheme(localSettings.value.theme)
    }
    
    await settingsStore.updateSettings(localSettings.value)
    isDirty.value = false
    console.log('Settings saved successfully')
  } catch (error) {
    console.error('Failed to save settings:', error)
  }
}

async function resetSettings() {
  try {
    await settingsStore.resetSettings()
    localSettings.value = { ...settingsStore.settings }
    isDirty.value = false
  } catch (error) {
    console.error('Failed to reset settings:', error)
  }
}

function handleClose() {
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
        <h2 class="modal-title">Settings</h2>
        <button class="modal-close" @click="handleClose">
          ✕
        </button>
      </div>

      <div class="modal-body">
        <div class="settings-tabs">
          <button
            class="tab-button"
            :class="{ active: activeTab === 'general' }"
            @click="activeTab = 'general'"
          >
            General
          </button>
          <button
            class="tab-button"
            :class="{ active: activeTab === 'connection' }"
            @click="activeTab = 'connection'"
          >
            Connection
          </button>
          <button
            class="tab-button"
            :class="{ active: activeTab === 'terminal' }"
            @click="activeTab = 'terminal'"
          >
            Terminal
          </button>
        </div>

        <div class="tab-content">
          <!-- General Settings -->
          <div v-if="activeTab === 'general'" class="settings-section">
            <div class="setting-group">
              <label class="setting-label">Theme</label>
              <select
                v-model="localSettings.theme"
                @change="handleThemeChange"
                class="setting-input"
              >
                <option v-for="theme in themesStore.availableThemes" :key="theme.metadata.id" :value="theme.metadata.id">
                  {{ theme.metadata.name }}
                </option>
              </select>
              <div class="setting-description">
                Choose the application theme. Current: {{ themesStore.currentTheme.metadata.name }}
              </div>
            </div>

            <div class="setting-group">
              <label class="setting-label">Default Protocol</label>
              <select
                v-model="localSettings.defaultProtocol"
                @change="markDirty"
                class="setting-input"
              >
                <option v-for="protocol in protocols" :key="protocol.value" :value="protocol.value">
                  {{ protocol.label }}
                </option>
              </select>
              <div class="setting-description">
                Default protocol when creating new sessions.
              </div>
            </div>
          </div>

          <!-- Connection Settings -->
          <div v-if="activeTab === 'connection'" class="settings-section">
            <div class="setting-group">
              <label class="setting-checkbox">
                <input
                  type="checkbox"
                  v-model="localSettings.autoConnect"
                  @change="markDirty"
                />
                <span class="checkbox-label">Auto-connect to sessions</span>
              </label>
              <div class="setting-description">
                Automatically connect to sessions when the application starts.
              </div>
            </div>

            <div class="setting-group">
              <label class="setting-checkbox">
                <input
                  type="checkbox"
                  v-model="localSettings.savePasswords"
                  @change="markDirty"
                />
                <span class="checkbox-label">Save passwords (encrypted)</span>
              </label>
              <div class="setting-description">
                Store passwords securely using Tauri's Stronghold plugin. Not recommended for production environments.
              </div>
            </div>

            <div class="setting-group">
              <label class="setting-label">Default SSH Key Path</label>
              <input
                type="text"
                v-model="localSettings.sshKeyPath"
                @input="markDirty"
                class="setting-input"
                placeholder="~/.ssh/id_rsa"
              />
              <div class="setting-description">
                Default path to SSH private key file for new sessions.
              </div>
            </div>
          </div>

          <!-- Terminal Settings -->
          <div v-if="activeTab === 'terminal'" class="settings-section">
            <div class="setting-group">
              <label class="setting-label">Terminal Theme</label>
              <select
                v-model="localSettings.terminalTheme"
                @change="markDirty"
                class="setting-input"
              >
                <option v-for="theme in terminalThemes" :key="theme.value" :value="theme.value">
                  {{ theme.label }}
                </option>
              </select>
              <div class="setting-description">
                Color scheme for terminal sessions.
              </div>
            </div>

            <div class="setting-group">
              <label class="setting-label">Font Size</label>
              <input
                type="number"
                v-model.number="localSettings.fontSize"
                @input="markDirty"
                class="setting-input"
                min="8"
                max="32"
              />
              <div class="setting-description">
                Font size for terminal text (8-32px).
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <div class="footer-left">
          <button
            @click="resetSettings"
            class="btn btn-secondary"
            :disabled="settingsStore.isLoading"
          >
            Reset to Defaults
          </button>
        </div>
        <div class="footer-right">
          <button
            @click="handleClose"
            class="btn btn-secondary"
            :disabled="settingsStore.isLoading"
          >
            Cancel
          </button>
          <button
            @click="saveSettings"
            class="btn btn-primary"
            :disabled="!isDirty || settingsStore.isLoading"
          >
            <span v-if="settingsStore.isLoading">Saving...</span>
            <span v-else>Save Changes</span>
          </button>
        </div>
      </div>
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
  max-width: 600px;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border-color);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.5rem 1.5rem 0;
  margin-bottom: 1rem;
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

.modal-body {
  flex: 1;
  overflow-y: auto;
  padding: 0 1.5rem;
}

.settings-tabs {
  display: flex;
  border-bottom: 1px solid var(--border-color);
  margin-bottom: 1.5rem;
}

.tab-button {
  padding: 0.75rem 1rem;
  border: none;
  background: none;
  color: var(--text-secondary);
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all 0.2s ease;
  font-weight: 500;
}

.tab-button:hover {
  color: var(--text-primary);
}

.tab-button.active {
  color: var(--text-accent);
  border-bottom-color: var(--text-accent);
}

.tab-content {
  min-height: 300px;
}

.settings-section {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.setting-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.setting-label {
  font-weight: 500;
  color: var(--text-primary);
  font-size: 0.875rem;
}

.setting-input {
  padding: 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.875rem;
  transition: border-color 0.2s ease;
}

.setting-input:focus {
  outline: none;
  border-color: var(--text-accent);
}

.setting-checkbox {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
}

.checkbox-label {
  font-weight: 500;
  color: var(--text-primary);
  font-size: 0.875rem;
}

.setting-description {
  color: var(--text-secondary);
  font-size: 0.75rem;
  line-height: 1.4;
}

.modal-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.5rem;
  border-top: 1px solid var(--border-color);
  margin-top: 1rem;
}

.footer-left,
.footer-right {
  display: flex;
  gap: 1rem;
}

@media (max-width: 600px) {
  .modal-overlay {
    padding: 0.5rem;
  }
  
  .modal-container {
    max-height: 95vh;
  }
  
  .settings-tabs {
    flex-direction: column;
  }
  
  .tab-button {
    text-align: left;
    border-bottom: none;
    border-left: 2px solid transparent;
  }
  
  .tab-button.active {
    border-left-color: var(--text-accent);
    border-bottom-color: transparent;
  }
  
  .modal-footer {
    flex-direction: column;
    gap: 1rem;
  }
  
  .footer-left,
  .footer-right {
    width: 100%;
    justify-content: stretch;
  }
}
</style>
