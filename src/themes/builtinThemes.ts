import type { Theme } from './types'

// Built-in light theme
export const lightTheme: Theme = {
  metadata: {
    id: 'light',
    name: 'Light',
    description: 'Clean and bright light theme',
    author: 'TermNest',
    version: '1.0.0',
    category: 'light',
    baseTheme: 'light'
  },
  colors: {
    // Background colors
    bgPrimary: '#ffffff',
    bgSecondary: '#f8f9fa',
    bgTertiary: '#e9ecef',
    bgQuaternary: '#dee2e6',
    
    // Text colors
    textPrimary: '#212529',
    textSecondary: '#6c757d',
    textAccent: '#0066cc',
    textMuted: '#adb5bd',
    
    // UI element colors
    borderColor: '#dee2e6',
    borderColorHover: '#adb5bd',
    shadow: '0 2px 4px rgba(0, 0, 0, 0.1)',
    shadowHover: '0 4px 8px rgba(0, 0, 0, 0.15)',
    
    // Status colors
    success: '#28a745',
    warning: '#ffc107',
    error: '#dc3545',
    info: '#17a2b8',
    
    // Terminal specific colors
    terminalBg: '#ffffff',
    terminalText: '#212529',
    terminalCursor: '#0066cc',
    
    // Button colors
    buttonPrimaryBg: '#0066cc',
    buttonPrimaryText: '#ffffff',
    buttonPrimaryHover: '#0052a3',
    buttonSecondaryBg: '#e9ecef',
    buttonSecondaryText: '#212529',
    buttonSecondaryHover: '#dee2e6',
    
    // Tab colors
    tabActiveBg: '#ffffff',
    tabInactiveBg: '#f8f9fa',
    tabBorderColor: '#dee2e6',
    
    // Status indicator colors
    statusConnected: '#28a745',
    statusConnecting: '#ffc107',
    statusDisconnected: '#6c757d',
    statusError: '#dc3545'
  }
}

// Built-in dark theme
export const darkTheme: Theme = {
  metadata: {
    id: 'dark',
    name: 'Dark',
    description: 'Comfortable dark theme for low-light environments',
    author: 'TermNest',
    version: '1.0.0',
    category: 'dark',
    baseTheme: 'dark'
  },
  colors: {
    // Background colors
    bgPrimary: '#1a1a1a',
    bgSecondary: '#2d2d2d',
    bgTertiary: '#404040',
    bgQuaternary: '#555555',
    
    // Text colors
    textPrimary: '#ffffff',
    textSecondary: '#b0b0b0',
    textAccent: '#4da6ff',
    textMuted: '#888888',
    
    // UI element colors
    borderColor: '#404040',
    borderColorHover: '#555555',
    shadow: '0 2px 4px rgba(0, 0, 0, 0.3)',
    shadowHover: '0 4px 8px rgba(0, 0, 0, 0.4)',
    
    // Status colors
    success: '#4caf50',
    warning: '#ff9800',
    error: '#f44336',
    info: '#2196f3',
    
    // Terminal specific colors
    terminalBg: '#1a1a1a',
    terminalText: '#ffffff',
    terminalCursor: '#4da6ff',
    
    // Button colors
    buttonPrimaryBg: '#4da6ff',
    buttonPrimaryText: '#ffffff',
    buttonPrimaryHover: '#3d8bff',
    buttonSecondaryBg: '#404040',
    buttonSecondaryText: '#ffffff',
    buttonSecondaryHover: '#555555',
    
    // Tab colors
    tabActiveBg: '#2d2d2d',
    tabInactiveBg: '#1a1a1a',
    tabBorderColor: '#404040',
    
    // Status indicator colors
    statusConnected: '#4caf50',
    statusConnecting: '#ff9800',
    statusDisconnected: '#888888',
    statusError: '#f44336'
  }
}

// Built-in system theme (follows OS preference)
export const systemTheme: Theme = {
  metadata: {
    id: 'system',
    name: 'System',
    description: 'Follows your operating system theme preference',
    author: 'TermNest',
    version: '1.0.0',
    category: 'light',
    baseTheme: 'light'
  },
  colors: {
    // Will be dynamically set based on system preference
    ...lightTheme.colors
  }
}

// Additional built-in themes
export const monokaiTheme: Theme = {
  metadata: {
    id: 'monokai',
    name: 'Monokai',
    description: 'Popular dark theme inspired by Sublime Text',
    author: 'TermNest',
    version: '1.0.0',
    category: 'dark',
    baseTheme: 'dark'
  },
  colors: {
    bgPrimary: '#272822',
    bgSecondary: '#3c3c3c',
    bgTertiary: '#49483e',
    bgQuaternary: '#5a5a5a',
    
    textPrimary: '#f8f8f2',
    textSecondary: '#cfcfc2',
    textAccent: '#a6e22e',
    textMuted: '#75715e',
    
    borderColor: '#49483e',
    borderColorHover: '#5a5a5a',
    shadow: '0 2px 4px rgba(0, 0, 0, 0.3)',
    shadowHover: '0 4px 8px rgba(0, 0, 0, 0.4)',
    
    success: '#a6e22e',
    warning: '#e6db74',
    error: '#f92672',
    info: '#66d9ef',
    
    terminalBg: '#272822',
    terminalText: '#f8f8f2',
    terminalCursor: '#a6e22e',
    
    buttonPrimaryBg: '#a6e22e',
    buttonPrimaryText: '#272822',
    buttonPrimaryHover: '#90c41f',
    buttonSecondaryBg: '#49483e',
    buttonSecondaryText: '#f8f8f2',
    buttonSecondaryHover: '#5a5a5a',
    
    tabActiveBg: '#3c3c3c',
    tabInactiveBg: '#272822',
    tabBorderColor: '#49483e',
    
    statusConnected: '#a6e22e',
    statusConnecting: '#e6db74',
    statusDisconnected: '#75715e',
    statusError: '#f92672'
  }
}

export const draculaTheme: Theme = {
  metadata: {
    id: 'dracula',
    name: 'Dracula',
    description: 'Dark theme with a touch of purple magic',
    author: 'TermNest',
    version: '1.0.0',
    category: 'dark',
    baseTheme: 'dark'
  },
  colors: {
    bgPrimary: '#282a36',
    bgSecondary: '#44475a',
    bgTertiary: '#6272a4',
    bgQuaternary: '#8be9fd',
    
    textPrimary: '#f8f8f2',
    textSecondary: '#bd93f9',
    textAccent: '#ff79c6',
    textMuted: '#6272a4',
    
    borderColor: '#44475a',
    borderColorHover: '#6272a4',
    shadow: '0 2px 4px rgba(0, 0, 0, 0.3)',
    shadowHover: '0 4px 8px rgba(0, 0, 0, 0.4)',
    
    success: '#50fa7b',
    warning: '#f1fa8c',
    error: '#ff5555',
    info: '#8be9fd',
    
    terminalBg: '#282a36',
    terminalText: '#f8f8f2',
    terminalCursor: '#ff79c6',
    
    buttonPrimaryBg: '#ff79c6',
    buttonPrimaryText: '#282a36',
    buttonPrimaryHover: '#ff6ac6',
    buttonSecondaryBg: '#44475a',
    buttonSecondaryText: '#f8f8f2',
    buttonSecondaryHover: '#6272a4',
    
    tabActiveBg: '#44475a',
    tabInactiveBg: '#282a36',
    tabBorderColor: '#6272a4',
    
    statusConnected: '#50fa7b',
    statusConnecting: '#f1fa8c',
    statusDisconnected: '#6272a4',
    statusError: '#ff5555'
  }
}

export const solarizedDarkTheme: Theme = {
  metadata: {
    id: 'solarized-dark',
    name: 'Solarized Dark',
    description: 'The classic Solarized dark theme',
    author: 'TermNest',
    version: '1.0.0',
    category: 'dark',
    baseTheme: 'dark'
  },
  colors: {
    bgPrimary: '#002b36',
    bgSecondary: '#073642',
    bgTertiary: '#586e75',
    bgQuaternary: '#657b83',
    
    textPrimary: '#839496',
    textSecondary: '#93a1a1',
    textAccent: '#268bd2',
    textMuted: '#586e75',
    
    borderColor: '#073642',
    borderColorHover: '#586e75',
    shadow: '0 2px 4px rgba(0, 0, 0, 0.3)',
    shadowHover: '0 4px 8px rgba(0, 0, 0, 0.4)',
    
    success: '#859900',
    warning: '#b58900',
    error: '#dc322f',
    info: '#268bd2',
    
    terminalBg: '#002b36',
    terminalText: '#839496',
    terminalCursor: '#268bd2',
    
    buttonPrimaryBg: '#268bd2',
    buttonPrimaryText: '#002b36',
    buttonPrimaryHover: '#2176c7',
    buttonSecondaryBg: '#073642',
    buttonSecondaryText: '#839496',
    buttonSecondaryHover: '#586e75',
    
    tabActiveBg: '#073642',
    tabInactiveBg: '#002b36',
    tabBorderColor: '#586e75',
    
    statusConnected: '#859900',
    statusConnecting: '#b58900',
    statusDisconnected: '#586e75',
    statusError: '#dc322f'
  }
}

export const builtInThemes: Theme[] = [
  lightTheme,
  darkTheme,
  systemTheme,
  monokaiTheme,
  draculaTheme,
  solarizedDarkTheme
]
