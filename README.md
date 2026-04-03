# Crush - Rush Configuration UI

![Rust](https://img.shields.io/badge/language-Rust-f74c00) ![License](https://img.shields.io/badge/license-Unlicense-green) ![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS-blue) ![Stay Amazing](https://img.shields.io/badge/Stay-Amazing-important)

Interactive TUI configuration app for [rush](https://github.com/isene/rush) shell. Browse and modify all rush settings with live color previews, theme selection, and a full 256-color palette.

Built on [crust](https://github.com/isene/crust).

## Features

- **7 setting categories**: Theme, Prompt Colors, Syntax Colors, Tab Colors, Completion, Behavior, Paths
- **Live color preview**: See color swatches next to each setting, full 256-color palette
- **Theme selection**: Cycle through 6 built-in themes (default, solarized, dracula, gruvbox, nord, monokai) with live prompt preview
- **Boolean toggles**: YES/NO with color indicators
- **Text editing**: Inline editing for paths and strings
- **Save prompt**: Warns on unsaved changes when quitting

## Quick Start

```bash
# Build
cargo build --release

# Run
./target/release/crush

# Or symlink to PATH
ln -s target/release/crush ~/bin/crush
```

## Keyboard

| Key | Action |
|-----|--------|
| j/k, UP/DOWN | Navigate settings |
| h/l, LEFT/RIGHT | Cycle values (colors, booleans, themes) |
| ENTER | Edit text/number values directly |
| W | Save config to ~/.rushrc.json |
| q/ESC | Quit (prompts to save if modified) |

## Settings Categories

### Theme
Select from 6 predefined color themes with live prompt preview showing username, hostname, directory, git branch, and command.

### Prompt Colors
Configure individual prompt element colors: username, hostname, directory, git branch, timestamp, prompt symbol.

### Syntax Colors
Configure syntax highlighting: commands, nicks, global nicks, paths, switches, bookmarks, colon commands, suggestions.

### Tab Colors
Configure tab completion and file type colors: selected tab, tab options, directories, executables, files.

### Completion
Toggle fuzzy matching, case sensitivity, metadata display. Set max completion results.

### Behavior
Toggle auto-correct, auto-pair, tips, right prompt. Set slow command threshold and autosave interval. Select history dedup strategy.

### Paths
Configure file manager path and other text settings.

## Part of the Fe2O3 Rust Terminal Suite

| Tool | Clones | Type |
|------|--------|------|
| [rush](https://github.com/isene/rush) | [rsh](https://github.com/isene/rsh) | Shell |
| **[crush](https://github.com/isene/crush)** | - | **Rush config UI** |
| [crust](https://github.com/isene/crust) | [rcurses](https://github.com/isene/rcurses) | TUI library |
| [pointer](https://github.com/isene/pointer) | [RTFM](https://github.com/isene/RTFM) | File manager |
| [plot](https://github.com/isene/plot) | [termchart](https://github.com/isene/termchart) | Charts |
| [glow](https://github.com/isene/glow) | [termpix](https://github.com/isene/termpix) | Image display |

## License

[Unlicense](https://unlicense.org/) - public domain.

## Credits

Created by Geir Isene (https://isene.org) with extensive pair-programming with Claude Code.
