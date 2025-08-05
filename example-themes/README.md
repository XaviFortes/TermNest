# Example Themes for TermNest

This directory contains example theme files that demonstrate how to create custom themes for TermNest.

## Using These Themes

To use any of these example themes:

1. Copy the desired `.json` file
2. Place it in your TermNest themes directory:
   - **Windows**: `%APPDATA%/TermNest/themes/`
   - **macOS**: `~/Library/Application Support/TermNest/themes/`
   - **Linux**: `~/.config/TermNest/themes/`
3. Restart TermNest or refresh the theme list
4. The theme will appear in Settings > General > Theme

## Available Example Themes

### `custom-theme-template.json`
A basic dark theme template that you can copy and modify to create your own themes.

### `sunset-orange.json`
A warm, sunset-inspired theme with orange and purple tones. Perfect for evening coding sessions.

### `forest-green.json`
A natural forest-inspired theme with various shades of green. Easy on the eyes for long coding sessions.

## Creating Your Own Themes

1. Start with the `custom-theme-template.json` as a base
2. Modify the colors to match your preferences
3. Update the metadata (id, name, description, author)
4. Save the file with a descriptive name
5. Place it in your themes directory

For detailed documentation on creating custom themes, see `../docs/themes.md`.

## Color Inspiration

When creating your own themes, consider these color palette resources:
- [Coolors.co](https://coolors.co/) - Color palette generator
- [Adobe Color](https://color.adobe.com/) - Professional color tools
- [Material Design Colors](https://materialui.co/colors/) - Google's Material Design palette
- [Dracula Theme](https://draculatheme.com/contribute) - Popular dark theme inspiration

## Tips

- Maintain good contrast between text and background colors
- Test your theme with different UI states (connected, disconnected, errors)
- Consider accessibility for users with visual impairments
- Use color temperature consistently (warm vs cool colors)
