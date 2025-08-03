# Password Authentication Implementation

## Overview
This implementation adds support for password-based SSH authentication to TermNest, alongside the existing public key and SSH agent authentication methods.

## Changes Made

### Backend (Rust)

#### 1. Updated SSH Configuration Structure
- Modified `SshConfig` struct in `ssh_new.rs` to use an `AuthMethod` enum instead of just a private key path
- Added support for three authentication methods:
  - `Password { password: String }`
  - `PublicKey { private_key_path: String }`
  - `Agent`

#### 2. Enhanced SSH Connection Logic
- Updated `SshManager::connect()` to handle different authentication methods
- Added password authentication using `session.userauth_password()`
- Added SSH agent authentication with identity iteration
- Maintained existing public key authentication

#### 3. New Tauri Commands
- Added `ssh_connect_with_password()` command for password authentication
- Maintains backward compatibility with existing `ssh_connect()` command

### Frontend (Vue.js)

#### 1. Password Dialog Component
- Created `PasswordDialog.vue` with:
  - Secure password input field
  - Connection information display
  - Error handling and loading states
  - Keyboard navigation (Enter to connect, Escape to cancel)
  - Accessible design with proper focus management

#### 2. Enhanced Terminal Component
- Updated `Terminal.vue` to:
  - Detect password authentication requirements
  - Show password dialog when needed
  - Handle password authentication flow
  - Maintain existing authentication methods

#### 3. Updated Configuration Handling
- Modified `getSessionConfig()` to support all authentication methods
- Added password prompt workflow
- Enhanced error handling for authentication failures

## Usage

### Creating a Password-Based Session
1. Open the "Create Session" dialog
2. Select "Password" as the authentication method
3. Save the session (password will be prompted during connection)

### Connecting with Password
1. Select a password-authenticated session
2. Click "Connect"
3. Enter your password in the prompted dialog
4. Connection will proceed automatically

## Security Considerations

### Password Handling
- Passwords are never stored on disk
- Passwords are only kept in memory during the authentication process
- Password input fields use proper `autocomplete="current-password"` attributes
- Password dialog auto-focuses for better UX

### Error Handling
- Authentication failures show appropriate error messages
- Network errors are distinguished from authentication errors
- Connection state is properly managed during authentication

## Testing

### Manual Testing Steps
1. Create a new session with password authentication
2. Attempt to connect and verify password dialog appears
3. Test with correct password (should connect successfully)
4. Test with incorrect password (should show error and allow retry)
5. Test canceling the password dialog (should abort connection)

### Public Key Authentication
- Verify existing public key authentication still works
- Test SSH agent authentication if available

## Future Enhancements

### Potential Improvements
1. Password caching for session duration (with user consent)
2. Integration with system credential managers
3. Support for keyboard-interactive authentication
4. Two-factor authentication support
5. Connection profiles with different auth methods

### Security Enhancements
1. Password strength indicators
2. Brute force protection
3. Session timeout for cached credentials
4. Audit logging for authentication attempts

## API Changes

### New Tauri Commands
```rust
ssh_connect_with_password(
    session_id: String,
    config: SshConfig,
    password: String,
    app_handle: AppHandle
) -> Result<(), String>
```

### Updated Types
```rust
pub enum AuthMethod {
    Password { password: String },
    PublicKey { private_key_path: String },
    Agent,
}

pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: AuthMethod,
}
```

## Error Codes

### Authentication Errors
- `"SSH authentication failed"` - Wrong credentials
- `"Connection failed: <error>"` - Network or SSH protocol errors
- `"Session not found"` - Invalid session ID
- `"Unsupported authentication method"` - Invalid auth method

### Dialog States
- `connecting` - Password dialog is shown and authenticating
- `error` - Authentication failed, showing error message
- `cancelled` - User cancelled authentication

This implementation provides a secure, user-friendly way to authenticate with SSH servers using passwords while maintaining all existing authentication methods.
