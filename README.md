# zellij-attention

Know which Zellij tab needs your attention — without checking each one.

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
</p>

A standalone Zellij WASM plugin that adds status icons directly to tab names. Works with both the default Zellij tab bar and [zjstatus](https://github.com/dj95/zjstatus). External tools can send status updates through `zellij action pipe`; the plugin renames the tab containing the target pane, for example `terminal` becomes `terminal ✓`.

This fork supports multi-state agent/activity indicators, watched panes, and focus-to-idle behavior while remaining agent-agnostic.

https://github.com/user-attachments/assets/646effc0-1c24-413d-bef3-3d85591cd89b

## Features

- **Tab-level status icons** — icons appended to tab names, visible at a glance
- **Multi-state activity** — thinking, shell, read/search, edit/write, subagent, web, other tool, waiting, done, idle
- **Watched panes** — mark a pane as tracked so it always falls back to idle (`○`)
- **Focus-to-idle** — focusing a watched pane/tab clears transient status back to idle instead of removing the icon
- **Explicit clear/unwatch** — clear status or stop tracking a pane via pipe events
- **Memory-only state** — lightweight, no disk I/O; stale panes cleaned up automatically
- **Configurable icons** — use any Unicode character or emoji as a status indicator
- **Standalone plugin** — works independently, no custom status bar required

## Installation

Download or build the plugin and add it to your Zellij config:

```bash
mkdir -p ~/.config/zellij/plugins
cp target/wasm32-wasip1/release/zellij-attention.wasm ~/.config/zellij/plugins/
```

Add to `~/.config/zellij/config.kdl`:

```kdl
load_plugins {
    "file:~/.config/zellij/plugins/zellij-attention.wasm" {
        enabled "true"
        thinking_icon "●"
        bash_icon "⚡"
        read_icon "◉"
        edit_icon "✎"
        subagent_icon "⊜"
        web_icon "◈"
        other_icon "⚙"
        waiting_icon "▶"
        completed_icon "✓"
        idle_icon "○"
        clear_on_tab_focus "true"
    }
}
```

The plugin loads in the background with no visible pane — it won't consume screen space.

> Existing Zellij sessions do not automatically reload `load_plugins`. Restart the session or run `start-or-reload-plugin` manually.

## Quick Start

Use `zellij action pipe` with an empty payload:

```bash
# Start tracking this pane and show idle
zellij action pipe --name "zellij-attention::watch::$ZELLIJ_PANE_ID" -- ""

# Show status
zellij action pipe --name "zellij-attention::thinking::$ZELLIJ_PANE_ID" -- ""
zellij action pipe --name "zellij-attention::completed::$ZELLIJ_PANE_ID" -- ""

# Focusing the tab/pane demotes completed/waiting/tool status back to idle for watched panes

# Stop tracking and remove icon
zellij action pipe --name "zellij-attention::unwatch::$ZELLIJ_PANE_ID" -- ""
```

For current sessions, reload the plugin with configuration:

```bash
zellij action start-or-reload-plugin \
  'file:/home/you/.config/zellij/plugins/zellij-attention.wasm' \
  --configuration 'enabled=true,thinking_icon=●,bash_icon=⚡,read_icon=◉,edit_icon=✎,subagent_icon=⊜,web_icon=◈,other_icon=⚙,waiting_icon=▶,completed_icon=✓,idle_icon=○,clear_on_tab_focus=true'
```

## Status Events

Pipe format:

```text
zellij-attention::EVENT_TYPE::PANE_ID
```

| Event type | Icon | Meaning |
| ---------- | ---- | ------- |
| `watch` | `○` | Track this pane and show idle fallback |
| `unwatch` | — | Stop tracking this pane and remove status |
| `clear` | `○` or — | Clear current status; watched panes fall back to idle, unwatched panes remove icon |
| `thinking` | `●` | Agent/process is thinking |
| `bash`, `running_bash`, `running-bash` | `⚡` | Running shell/Bash command |
| `read`, `reading`, `search`, `searching` | `◉` | Reading/searching files |
| `edit`, `editing`, `write`, `writing` | `✎` | Editing/writing files |
| `subagent`, `spawning_subagent`, `spawning-subagent` | `⊜` | Spawning or waiting on a subagent |
| `web`, `web_search`, `web-search`, `fetch` | `◈` | Web search/fetch |
| `other`, `tool`, `other_tool`, `other-tool` | `⚙` | Other tool activity |
| `waiting`, `prompt`, `user_prompt`, `user-prompt` | `▶` | Waiting for user prompt/input |
| `completed`, `done` | `✓` | Task done |
| `idle` | `○` | Idle |

`PANE_ID` is the numeric pane ID, usually available as `$ZELLIJ_PANE_ID` inside a pane.

> Use `--name` broadcast pipes. Do not use `--plugin` for normal updates, because targeted pipes can launch extra plugin instances.

## Watched Pane Semantics

Watched panes let external agents advertise their presence even when idle.

```text
watch      my-tab ○
thinking   my-tab ●
completed  my-tab ✓
focus tab  my-tab ○
unwatch    my-tab
```

This makes the plugin agent-agnostic: the plugin only stores status; an integration such as a Pi extension, Claude hook, shell wrapper, or any other process decides when to send each event.

## Configuration

All configuration is optional.

| Option | Default | Description |
| ------ | ------- | ----------- |
| `enabled` | `"true"` | Enable or disable notifications |
| `thinking_icon` | `"●"` | Thinking icon |
| `bash_icon` | `"⚡"` | Running shell/Bash icon |
| `read_icon` | `"◉"` | Reading/searching icon |
| `edit_icon` | `"✎"` | Editing/writing icon |
| `subagent_icon` | `"⊜"` | Subagent icon |
| `web_icon` | `"◈"` | Web search/fetch icon |
| `other_icon` | `"⚙"` | Other tool icon |
| `waiting_icon` | `"▶"` | Waiting for user input icon |
| `completed_icon` | `"✓"` | Completed/done icon |
| `idle_icon` | `"○"` | Idle fallback icon |
| `clear_on_tab_focus` | `"true"` | Clear/demote statuses when the tab becomes active |

Icons are appended to the end of tab names, e.g. `terminal ✓`. Tab names are plain text; per-icon colors are not supported by Zellij tab renaming.

## Shell Helpers

```bash
zellij_attention() {
    local event="$1"
    local pane_id="${2:-$ZELLIJ_PANE_ID}"
    [ -z "$pane_id" ] && echo "Not in Zellij" && return 1
    zellij action pipe --name "zellij-attention::${event}::${pane_id}" -- ""
}

zellij_attention watch
zellij_attention thinking
zellij_attention completed
zellij_attention clear
zellij_attention unwatch
```

## Development

```bash
# Build
cargo build --target wasm32-wasip1 --release

# Install
cp target/wasm32-wasip1/release/zellij-attention.wasm ~/.config/zellij/plugins/

# Debug build (enables verbose logging)
cargo build --target wasm32-wasip1
tail -f /tmp/zellij-*/zellij-log-*/zellij.log | grep "zellij-attention"
```

## Troubleshooting

See [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for common issues and solutions.

## License

MIT
