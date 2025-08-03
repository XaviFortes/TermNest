# TermNest Multi-Session Features

## Overview

TermNest now supports multiple concurrent sessions with a modern tabbed interface. Users can open, manage, and switch between multiple SSH connections seamlessly.

## Key Features

### 🔄 Multi-Session Support
- **Multiple Active Sessions**: Open several SSH connections simultaneously
- **Tab-Based Interface**: Modern browser-like tabs for easy session switching
- **Session State Management**: Each session maintains its own terminal state

### 🎯 Modern UI/UX
- **Sidebar Navigation**: Collapsible sidebar showing all available sessions
- **Quick Session Selector**: Modal to quickly open new sessions
- **Real-time Status**: Visual indicators for connection status (connected, connecting, error)
- **Session Information**: Display session name, host, and connection details

### ⌨️ Keyboard Shortcuts
- `Ctrl + Shift + P`: Open Quick Actions menu
- `Ctrl + N`: Create new session
- `Ctrl + W`: Close current tab
- `Ctrl + Shift + W`: Close all tabs
- `Ctrl + B`: Toggle sidebar
- `Ctrl + Tab`: Next tab
- `Ctrl + Shift + Tab`: Previous tab
- `Escape`: Close menus

### 🎛️ Advanced Management
- **Context Menu**: Right-click on sessions for additional options
- **Session Duplication**: Clone existing sessions
- **Session Export**: Export session configurations as JSON
- **Drag & Drop**: Intuitive session organization
- **Search & Filter**: Find sessions quickly

### 📊 Status Monitoring
- **Active Sessions Counter**: See how many sessions are running
- **Connection Status**: Real-time connection monitoring
- **Memory Usage**: Track application memory consumption
- **Live Clock**: Current time display
- **Loading Indicators**: Visual feedback for ongoing operations

## User Interface

### Sidebar Layout
- **Sessions List**: Shows all configured sessions with status indicators
- **Search Box**: Filter sessions by name, host, or username
- **Add Session Button**: Quick access to create new sessions
- **Collapsible Design**: Save screen space when needed

### Tab Interface
- **Session Tabs**: Each active session gets its own tab
- **Status Indicators**: Color-coded dots showing connection status
- **Close Buttons**: Individual tab closing
- **Session Selector**: Add new sessions via modal

### Quick Actions Menu
- **Recent Sessions**: Quick access to recently used sessions
- **Navigation Commands**: Tab switching and sidebar controls
- **Session Management**: Create, close, and manage sessions

## Session States

### Connection Status
- 🟢 **Connected**: Session is active and ready
- 🟡 **Connecting**: Connection in progress
- 🔴 **Error**: Connection failed or lost
- ⚪ **Disconnected**: Session not connected

### Session Management
- **Active**: Session has an open tab
- **Inactive**: Session exists but no active tab
- **Recent**: Recently used sessions for quick access

## Performance
- **Lazy Loading**: Terminals are created only when needed
- **Memory Efficient**: Sessions are properly cleaned up when closed
- **Fast Switching**: Instant tab switching with cached states

## Compatibility
- Works with all existing session types (SSH, SFTP, etc.)
- Maintains backward compatibility with single-session workflows
- Supports all authentication methods (password, key, agent)

## Tips
1. Use `Ctrl + Shift + P` to access all features quickly
2. Right-click sessions for advanced options
3. Collapse the sidebar to maximize terminal space
4. Export session configurations for backup or sharing
5. Use keyboard shortcuts for faster navigation
