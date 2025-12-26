---
title: "Keyboard Shortcuts"
layout: doc
---

<!-- AUTO-GENERATED FROM src/ui/keybindings.rs - DO NOT EDIT MANUALLY -->
<!-- Regenerate with: cargo run -- docs -->

# Keyboard Shortcuts

Operator uses vim-style keybindings for navigation and actions. This reference documents all available keyboard shortcuts.

## Quick Reference

| Key | Action | Context |
| --- | --- | --- |
| `q` | Quit Operator | Dashboard |
| `?` | Toggle help | Dashboard |
| `Tab` | Switch between panels | Dashboard |
| `j/↓` | Move down | Dashboard |
| `k/↑` | Move up | Dashboard |
| `Q` | Focus Queue panel | Dashboard |
| `A/a` | Focus Agents panel | Dashboard |
| `Enter` | Select / Confirm | Dashboard |
| `Esc` | Cancel / Close | Dashboard |
| `L/l` | Launch selected ticket | Dashboard |
| `P/p` | Pause queue processing | Dashboard |
| `R/r` | Resume queue processing | Dashboard |
| `S` | Manual sync (rate limits + sessions) | Dashboard |
| `W/w` | Toggle Backstage server | Dashboard |
| `V/v` | Show session preview | Dashboard |
| `C` | Create new ticket | Dashboard |
| `J` | Open Projects menu | Dashboard |
| `g` | Scroll to top | Session Preview |
| `G` | Scroll to bottom | Session Preview |
| `PgUp` | Page up | Session Preview |
| `PgDn` | Page down | Session Preview |
| `Esc/q` | Close preview | Session Preview |
| `Y/y` | Launch agent | Launch Dialog |
| `V/v` | View ticket ($VISUAL or open) | Launch Dialog |
| `E/e` | Edit ticket ($EDITOR) | Launch Dialog |
| `N/n` | Cancel | Launch Dialog |

## Dashboard

These shortcuts are available in the main dashboard view.

### General

| Key | Action |
| --- | --- |
| `q` | Quit Operator |
| `?` | Toggle help |

### Navigation

| Key | Action |
| --- | --- |
| `Tab` | Switch between panels |
| `j/↓` | Move down |
| `k/↑` | Move up |
| `Q` | Focus Queue panel |
| `A/a` | Focus Agents panel |

### Actions

| Key | Action |
| --- | --- |
| `Enter` | Select / Confirm |
| `Esc` | Cancel / Close |
| `L/l` | Launch selected ticket |
| `P/p` | Pause queue processing |
| `R/r` | Resume queue processing |
| `S` | Manual sync (rate limits + sessions) |
| `W/w` | Toggle Backstage server |
| `V/v` | Show session preview |

### Dialogs

| Key | Action |
| --- | --- |
| `C` | Create new ticket |
| `J` | Open Projects menu |

## Session Preview

These shortcuts are available when viewing a session preview.

### Navigation

| Key | Action |
| --- | --- |
| `g` | Scroll to top |
| `G` | Scroll to bottom |
| `PgUp` | Page up |
| `PgDn` | Page down |

### Actions

| Key | Action |
| --- | --- |
| `Esc/q` | Close preview |

## Launch Dialog

These shortcuts are available in the ticket launch confirmation dialog.

### Actions

| Key | Action |
| --- | --- |
| `Y/y` | Launch agent |
| `V/v` | View ticket ($VISUAL or open) |
| `E/e` | Edit ticket ($EDITOR) |
| `N/n` | Cancel |

