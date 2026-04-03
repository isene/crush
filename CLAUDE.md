# Crush - Rush Configuration UI

Interactive TUI configuration app for [rush](https://github.com/isene/rush) shell. Built on crust.

## Build

```bash
PATH="/usr/bin:$PATH" cargo build --release
```

## Usage

```bash
crush          # Opens TUI config editor for ~/.rushrc.json
```

## Keys

- j/k or UP/DOWN: Navigate settings
- h/l or LEFT/RIGHT: Cycle values (colors, booleans, themes)
- ENTER: Edit text/number values
- W: Save config
- q/ESC: Quit (prompts to save if modified)
