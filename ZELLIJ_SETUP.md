# Zellij Plugin Setup

## Prerequisites

Install rustup (required for WASM target):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Build the Plugin

```bash
./build-plugin.sh
```

This will:
1. Add the `wasm32-wasip1` target
2. Compile the plugin to WASM
3. Install to `~/.config/zellij/plugins/`

## Usage

### Quick Test
```bash
zellij action new-pane --plugin file:~/.config/zellij/plugins/faster.wasm
```

### Add to Layout

Create `~/.config/zellij/layouts/faster.kdl`:
```kdl
layout {
    pane size=1 borderless=true {
        plugin location="tab-bar"
    }

    pane split_direction="vertical" {
        pane size="70%" {
            // Your main terminal
        }
        pane size="30%" {
            plugin location="file:~/.config/zellij/plugins/faster.wasm"
        }
    }

    pane size=2 borderless=true {
        plugin location="status-bar"
    }
}
```

Then run:
```bash
zellij --layout faster
```

## Keyboard Shortcuts

In the plugin pane:
- `i` - Enter command input mode
- `Esc` - Exit input mode
- `Enter` - Submit command
- `j/k` or `↓/↑` - Navigate tasks
- `d` - Cancel selected task
- `r` - Refresh task list

## Architecture

The plugin displays the same SQLite queue used by the `faster` CLI:
- Read-only view of `~/.faster/knowledge.db`
- Real-time updates every second
- Keyboard-driven interface
- No mouse required

Commands added via the plugin are processed by `faster daemon`.
