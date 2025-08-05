# TermNest Theme System

TermNest features a powerful and flexible theme system that allows you to customize the appearance of the application and easily create your own themes.

## Built-in Themes

TermNest comes with several built-in themes:

- **Light**: A clean and bright theme for well-lit environments
- **Dark**: A comfortable dark theme for low-light environments  
- **System**: Automatically follows your operating system's theme preference
- **Monokai**: A popular dark theme inspired by Sublime Text
- **Dracula**: A dark theme with purple accents

## Changing Themes

1. Click the "Settings" button in the top-right corner
2. In the General tab, select your preferred theme from the dropdown
3. The theme will be applied immediately

## Creating Custom Themes

You can create your own custom themes by placing JSON files in the themes directory. TermNest will automatically load these themes on startup.

### Theme File Location

Custom themes should be placed in:
- **Windows**: `%APPDATA%/TermNest/themes/`
- **macOS**: `~/Library/Application Support/TermNest/themes/`
- **Linux**: `~/.config/TermNest/themes/`

### Theme File Structure

Each theme file should be a JSON file with the following structure:

```json
{
  "metadata": {
    "id": "my-custom-theme",
    "name": "My Custom Theme",
    "description": "A beautiful custom theme",
    "author": "Your Name",
    "version": "1.0.0",
    "category": "dark",
    "baseTheme": "dark"
  },
  "colors": {
    "bgPrimary": "#1a1a1a",
    "bgSecondary": "#2d2d2d",
    "bgTertiary": "#404040",
    "bgQuaternary": "#555555",
    
    "textPrimary": "#ffffff",
    "textSecondary": "#b0b0b0",
    "textAccent": "#4da6ff",
    "textMuted": "#888888",
    
    "borderColor": "#404040",
    "borderColorHover": "#555555",
    "shadow": "0 2px 4px rgba(0, 0, 0, 0.3)",
    "shadowHover": "0 4px 8px rgba(0, 0, 0, 0.4)",
    
    "success": "#4caf50",
    "warning": "#ff9800",
    "error": "#f44336",
    "info": "#2196f3",
    
    "terminalBg": "#1a1a1a",
    "terminalText": "#ffffff",
    "terminalCursor": "#4da6ff",
    
    "buttonPrimaryBg": "#4da6ff",
    "buttonPrimaryText": "#ffffff",
    "buttonPrimaryHover": "#3d8bff",
    "buttonSecondaryBg": "#404040",
    "buttonSecondaryText": "#ffffff",
    "buttonSecondaryHover": "#555555",
    
    "tabActiveBg": "#2d2d2d",
    "tabInactiveBg": "#1a1a1a",
    "tabBorderColor": "#404040",
    
    "statusConnected": "#4caf50",
    "statusConnecting": "#ff9800",
    "statusDisconnected": "#888888",
    "statusError": "#f44336"
  }
}
```

### Metadata Fields

- **id**: Unique identifier for the theme (required)
- **name**: Display name for the theme (required)
- **description**: Brief description of the theme (optional)
- **author**: Your name or username (optional)
- **version**: Version number (optional)
- **category**: Theme category - "light", "dark", "colorful", "minimal", or "custom" (optional)
- **baseTheme**: Base theme type - "light" or "dark" (optional)

### Color Properties

All color properties support standard CSS color formats (hex, rgb, rgba, hsl, etc.):

#### Background Colors
- **bgPrimary**: Main background color
- **bgSecondary**: Secondary background (sidebars, headers)
- **bgTertiary**: Tertiary background (cards, modals)
- **bgQuaternary**: Fourth level background (optional)

#### Text Colors
- **textPrimary**: Main text color
- **textSecondary**: Secondary text color
- **textAccent**: Accent text color (links, highlights)
- **textMuted**: Muted text color (optional)

#### UI Elements
- **borderColor**: Default border color
- **borderColorHover**: Border color on hover (optional)
- **shadow**: Default shadow
- **shadowHover**: Shadow on hover

#### Status Colors
- **success**: Success state color
- **warning**: Warning state color
- **error**: Error state color
- **info**: Information state color

#### Terminal Colors (optional)
- **terminalBg**: Terminal background
- **terminalText**: Terminal text
- **terminalCursor**: Terminal cursor

#### Button Colors
- **buttonPrimaryBg**: Primary button background
- **buttonPrimaryText**: Primary button text
- **buttonPrimaryHover**: Primary button hover background
- **buttonSecondaryBg**: Secondary button background
- **buttonSecondaryText**: Secondary button text
- **buttonSecondaryHover**: Secondary button hover background

#### Tab Colors (optional)
- **tabActiveBg**: Active tab background
- **tabInactiveBg**: Inactive tab background
- **tabBorderColor**: Tab border color

#### Status Indicators
- **statusConnected**: Connected status color
- **statusConnecting**: Connecting status color
- **statusDisconnected**: Disconnected status color
- **statusError**: Error status color

## Example Themes

### Ocean Blue Theme
```json
{
  "metadata": {
    "id": "ocean-blue",
    "name": "Ocean Blue",
    "description": "A calming ocean-inspired theme",
    "author": "TermNest",
    "version": "1.0.0",
    "category": "colorful",
    "baseTheme": "dark"
  },
  "colors": {
    "bgPrimary": "#0f1419",
    "bgSecondary": "#1e2328",
    "bgTertiary": "#272e33",
    "textPrimary": "#ffffff",
    "textSecondary": "#b3c5d1",
    "textAccent": "#39bae6",
    "borderColor": "#272e33",
    "shadow": "0 2px 4px rgba(0, 0, 0, 0.3)",
    "shadowHover": "0 4px 8px rgba(0, 0, 0, 0.4)",
    "success": "#87d96c",
    "warning": "#f2cc60",
    "error": "#f07178",
    "info": "#39bae6",
    "buttonPrimaryBg": "#39bae6",
    "buttonPrimaryText": "#0f1419",
    "buttonPrimaryHover": "#2aa9d6",
    "buttonSecondaryBg": "#272e33",
    "buttonSecondaryText": "#ffffff",
    "buttonSecondaryHover": "#34424a",
    "statusConnected": "#87d96c",
    "statusConnecting": "#f2cc60",
    "statusDisconnected": "#6b8794",
    "statusError": "#f07178"
  }
}
```

### Minimal Light Theme
```json
{
  "metadata": {
    "id": "minimal-light",
    "name": "Minimal Light",
    "description": "A clean, minimal light theme",
    "author": "TermNest",
    "version": "1.0.0",
    "category": "minimal",
    "baseTheme": "light"
  },
  "colors": {
    "bgPrimary": "#ffffff",
    "bgSecondary": "#fafafa",
    "bgTertiary": "#f5f5f5",
    "textPrimary": "#333333",
    "textSecondary": "#666666",
    "textAccent": "#007acc",
    "borderColor": "#e0e0e0",
    "shadow": "0 1px 3px rgba(0, 0, 0, 0.1)",
    "shadowHover": "0 2px 6px rgba(0, 0, 0, 0.15)",
    "success": "#4caf50",
    "warning": "#ff9800",
    "error": "#f44336",
    "info": "#2196f3",
    "buttonPrimaryBg": "#007acc",
    "buttonPrimaryText": "#ffffff",
    "buttonPrimaryHover": "#005999",
    "buttonSecondaryBg": "#f5f5f5",
    "buttonSecondaryText": "#333333",
    "buttonSecondaryHover": "#e0e0e0",
    "statusConnected": "#4caf50",
    "statusConnecting": "#ff9800",
    "statusDisconnected": "#666666",
    "statusError": "#f44336"
  }
}
```

## Tips for Creating Themes

1. **Start with an existing theme**: Copy a built-in theme and modify the colors
2. **Maintain contrast**: Ensure sufficient contrast between text and background colors
3. **Test thoroughly**: Test your theme with different UI states (connected, disconnected, errors)
4. **Use color tools**: Online color palette generators can help create cohesive themes
5. **Consider accessibility**: Ensure your theme is accessible to users with visual impairments

## Sharing Themes

You can share your custom themes by:
1. Saving the theme JSON file
2. Sharing the file with other users
3. Having them place it in their themes directory
4. Restarting TermNest or refreshing the theme list

The theme will appear in the Settings > General > Theme dropdown.

## Troubleshooting

### Theme not appearing
- Check that the JSON file is valid (use a JSON validator)
- Ensure the file is in the correct themes directory
- Restart TermNest
- Check the console for error messages

### Colors not applying
- Verify all required color properties are present
- Check for typos in color property names
- Ensure color values are valid CSS colors

### Theme looks broken
- Check that `baseTheme` is set to "light" or "dark"
- Ensure sufficient contrast between text and background
- Verify all UI element colors are defined
