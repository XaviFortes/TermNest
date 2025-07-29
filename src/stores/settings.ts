import { defineStore } from 'pinia'
import { ref } from 'vue'
import { Store } from '@tauri-apps/plugin-store'

export interface AppSettings {
  theme: 'light' | 'dark' | 'system'
  defaultProtocol: 'SSH' | 'SFTP'
  autoConnect: boolean
  savePasswords: boolean
  sshKeyPath: string
  terminalTheme: string
  fontSize: number
}

const defaultSettings: AppSettings = {
  theme: 'system',
  defaultProtocol: 'SSH',
  autoConnect: false,
  savePasswords: false,
  sshKeyPath: '~/.ssh/id_rsa',
  terminalTheme: 'dark',
  fontSize: 14,
}

export const useSettingsStore = defineStore('settings', () => {
  // State
  const settings = ref<AppSettings>({ ...defaultSettings })
  const isLoading = ref(false)
  const store = ref<Store | null>(null)

  // Actions
  async function initializeStore() {
    try {
      store.value = await Store.load('.settings.dat')
      await loadSettings()
    } catch (error) {
      console.error('Failed to initialize settings store:', error)
    }
  }

  async function loadSettings() {
    if (!store.value) {
      await initializeStore()
    }

    try {
      isLoading.value = true
      
      // Load each setting individually with fallbacks
      const savedSettings: Partial<AppSettings> = {}
      
      for (const key of Object.keys(defaultSettings)) {
        try {
          const value = await store.value?.get(key)
          if (value !== null && value !== undefined) {
            (savedSettings as any)[key] = value
          }
        } catch (error) {
          console.warn(`Failed to load setting ${key}:`, error)
        }
      }
      
      settings.value = { ...defaultSettings, ...savedSettings }
    } catch (error) {
      console.error('Failed to load settings:', error)
      settings.value = { ...defaultSettings }
    } finally {
      isLoading.value = false
    }
  }

  async function updateSetting<K extends keyof AppSettings>(
    key: K,
    value: AppSettings[K]
  ) {
    if (!store.value) {
      await initializeStore()
    }

    try {
      settings.value[key] = value
      await store.value?.set(key, value)
      await store.value?.save()
    } catch (error) {
      console.error(`Failed to update setting ${key}:`, error)
      throw error
    }
  }

  async function updateSettings(newSettings: Partial<AppSettings>) {
    if (!store.value) {
      await initializeStore()
    }

    try {
      for (const [key, value] of Object.entries(newSettings)) {
        if (value !== undefined) {
          (settings.value as any)[key] = value
          await store.value?.set(key, value)
        }
      }
      await store.value?.save()
    } catch (error) {
      console.error('Failed to update settings:', error)
      throw error
    }
  }

  async function resetSettings() {
    if (!store.value) {
      await initializeStore()
    }

    try {
      await store.value?.clear()
      await store.value?.save()
      settings.value = { ...defaultSettings }
    } catch (error) {
      console.error('Failed to reset settings:', error)
      throw error
    }
  }

  return {
    // State
    settings,
    isLoading,
    // Actions
    initializeStore,
    loadSettings,
    updateSetting,
    updateSettings,
    resetSettings,
  }
})
