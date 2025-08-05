import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Theme, ThemeColors } from '../themes/types'
import { builtInThemes, systemTheme, lightTheme, darkTheme } from '../themes/builtinThemes'

export const useThemesStore = defineStore('themes', () => {
  // State
  const currentTheme = ref<Theme>(lightTheme)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // Computed
  const availableThemes = computed(() => builtInThemes)
  const currentThemeId = computed(() => currentTheme.value.metadata.id)

  // Actions
  async function initializeThemes() {
    try {
      isLoading.value = true
      
      // Set initial theme based on saved preference or system
      const savedThemeId = getSavedThemeId()
      console.log('Initializing themes with saved theme:', savedThemeId)
      await setTheme(savedThemeId || 'system')
      
      // Watch for system theme changes if using system theme
      if (currentTheme.value.metadata.id === 'system') {
        watchSystemTheme()
      }
    } catch (err) {
      console.error('Failed to initialize themes:', err)
      error.value = err as string
    } finally {
      isLoading.value = false
    }
  }

  async function setTheme(themeId: string) {
    console.log('Setting theme to:', themeId)
    const theme = availableThemes.value.find(t => t.metadata.id === themeId)
    
    if (!theme) {
      console.warn(`Theme not found: ${themeId}`, 'Available themes:', availableThemes.value.map(t => t.metadata.id))
      return
    }

    console.log('Found theme:', theme.metadata.name, 'Colors:', theme.colors)

    // Handle system theme
    if (themeId === 'system') {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
      currentTheme.value = {
        ...systemTheme,
        colors: prefersDark ? darkTheme.colors : lightTheme.colors
      }
      console.log('System theme applied, using:', prefersDark ? 'dark' : 'light')
    } else {
      currentTheme.value = theme
      console.log('Custom theme applied:', theme.metadata.name)
    }

    // Apply theme to DOM
    applyThemeToDOM(currentTheme.value.colors)
    
    // Save theme preference
    saveThemeId(themeId)
    
    console.log('Theme setting completed for:', themeId)
  }

  function applyThemeToDOM(colors: ThemeColors) {
    console.log('Applying theme to DOM with colors:', colors)
    const root = document.documentElement
    
    // Apply CSS custom properties
    Object.entries(colors).forEach(([key, value]) => {
      const cssVar = camelToKebab(key)
      root.style.setProperty(`--${cssVar}`, value as string)
      console.log(`Set --${cssVar}: ${value}`)
    })
    
    // Apply theme class for any CSS that might still need it
    const body = document.body
    body.className = body.className.replace(/theme-\w+/g, '')
    body.classList.add(`theme-${currentTheme.value.metadata.baseTheme || 'light'}`)
    
    console.log(`Applied theme: ${currentTheme.value.metadata.name}`)
    console.log(`Theme class: theme-${currentTheme.value.metadata.baseTheme || 'light'}`)
    
    // Verify some key variables were set
    const computedStyle = getComputedStyle(root)
    console.log('Verification - bg-primary:', computedStyle.getPropertyValue('--bg-primary'))
    console.log('Verification - text-primary:', computedStyle.getPropertyValue('--text-primary'))
    console.log('Verification - text-accent:', computedStyle.getPropertyValue('--text-accent'))
  }

  function watchSystemTheme() {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    
    const updateSystemTheme = () => {
      if (currentTheme.value.metadata.id === 'system') {
        const colors = mediaQuery.matches ? darkTheme.colors : lightTheme.colors
        currentTheme.value = {
          ...systemTheme,
          colors
        }
        applyThemeToDOM(colors)
      }
    }

    mediaQuery.addEventListener('change', updateSystemTheme)
    
    return () => mediaQuery.removeEventListener('change', updateSystemTheme)
  }

  // Helper functions
  function getSavedThemeId(): string | null {
    try {
      return localStorage.getItem('termnest-theme')
    } catch {
      return null
    }
  }

  function saveThemeId(themeId: string): void {
    try {
      localStorage.setItem('termnest-theme', themeId)
    } catch (err) {
      console.warn('Failed to save theme preference:', err)
    }
  }

  function camelToKebab(str: string): string {
    return str.replace(/([a-z0-9]|(?=[A-Z]))([A-Z])/g, '$1-$2').toLowerCase()
  }

  return {
    // State
    currentTheme,
    isLoading,
    error,
    
    // Computed
    availableThemes,
    currentThemeId,
    
    // Actions
    initializeThemes,
    setTheme,
    applyThemeToDOM,
    
    // Debug function
    debugThemes: () => {
      console.log('Available themes:', availableThemes.value.map(t => t.metadata))
      console.log('Current theme:', currentTheme.value.metadata)
      console.log('Current CSS vars:', [...document.documentElement.style])
    }
  }
})
