# termsearch

[![pipeline](https://github.com/zenoxygen/termsearch/actions/workflows/ci.yaml/badge.svg)](https://github.com/zenoxygen/termsearch/actions/workflows/ci.yaml)
[![crates.io](https://img.shields.io/crates/v/termsearch.svg)](https://crates.io/crates/termsearch)

A minimalist and super fast terminal history search tool, that uses a weighted scoring system to rank commands.

- Recency: more recent commands are given higher priority
- Frequency: commands used more frequently are given higher priority

*Note: it only works with `zsh` on Linux for now.*

## Usage

### Initialize termsearch

```bash
termsearch init
```

This command does the following:

- Creates a `termsearch.zsh` file in the ZSH configuration directory.
- Appends a command to the `~/.zshrc` file, which will load `termsearch.zsh` for a new terminal session.
- Rebinds the **Ctrl+R** key (the default keybinding) in the ZSH shell to use `termsearch`.

*Restart the terminal to enable it.*

### Search for a command

```
termsearch search
```

- **Up/Down** and **Shift+Tab/Tab** navigate up/down through the search results.
- **Enter** selects the highlighted command and pastes it into the terminal's input line.
- **Ctrl+C**, **Ctrl+D**, **Esc** cancel the search.

### Options

```
-m, --max-history <MAX_HISTORY>  Maximum number of history lines to read [default: 10000]
-r, --max-results <MAX_RESULTS>  Maximum number of results to display [default: 10]
```

## Installation

### From crates.io (recommended)

```bash
cargo install termsearch
```

### From source

```bash
cargo install --path . --locked
```

## Uninstallation

```bash
cargo uninstall termsearch
```

## Debug

Set log level with `TERMSEARCH_LOG` environment variable:

```bash
export TERMSEARCH_LOG=debug
```
