// Theme interfaces and types
export interface ThemeColors {
  // Background colors
  bgPrimary: string
  bgSecondary: string
  bgTertiary: string
  bgQuaternary?: string
  
  // Text colors
  textPrimary: string
  textSecondary: string
  textAccent: string
  textMuted?: string
  
  // UI element colors
  borderColor: string
  borderColorHover?: string
  shadow: string
  shadowHover: string
  
  // Status colors
  success: string
  warning: string
  error: string
  info: string
  
  // Terminal specific colors
  terminalBg?: string
  terminalText?: string
  terminalCursor?: string
  
  // Button colors
  buttonPrimaryBg: string
  buttonPrimaryText: string
  buttonPrimaryHover: string
  buttonSecondaryBg: string
  buttonSecondaryText: string
  buttonSecondaryHover: string
  
  // Tab colors
  tabActiveBg?: string
  tabInactiveBg?: string
  tabBorderColor?: string
  
  // Status indicator colors
  statusConnected: string
  statusConnecting: string
  statusDisconnected: string
  statusError: string
}

export interface ThemeMetadata {
  id: string
  name: string
  description?: string
  author?: string
  version?: string
  category?: 'light' | 'dark' | 'colorful' | 'minimal' | 'custom'
  baseTheme?: 'light' | 'dark'
}

export interface Theme {
  metadata: ThemeMetadata
  colors: ThemeColors
}

export interface UserTheme extends Theme {
  isUserCreated: boolean
  filePath?: string
}
