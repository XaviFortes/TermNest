# TermNest Themes

This directory contains the theme system for TermNest.

## Structure

- `types.ts` - TypeScript interfaces and types for themes
- `builtinThemes.ts` - Built-in themes (Light, Dark, System, Monokai, Dracula)
- `../stores/themes.ts` - Theme store for managing and applying themes

## Usage

The theme system is automatically initialized when the app starts. Themes are applied by setting CSS custom properties on the document root.

## Creating New Built-in Themes

To add a new built-in theme:

1. Add the theme object to `builtinThemes.ts`
2. Export it in the `builtInThemes` array
3. Follow the `Theme` interface defined in `types.ts`

## Custom User Themes

Users can create custom themes by placing JSON files in their app data directory:
- Windows: `%APPDATA%/TermNest/themes/`
- macOS: `~/Library/Application Support/TermNest/themes/`
- Linux: `~/.config/TermNest/themes/`

See `../docs/themes.md` for detailed documentation on creating custom themes.
