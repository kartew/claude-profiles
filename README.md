# ccp - Claude Code Profiles

CLI tool for managing Claude Code settings profiles, similar to AWS CLI profiles.

## Installation

```bash
cargo build --release
cp target/release/ccp ~/.local/bin/
# or
sudo cp target/release/ccp /usr/local/bin/
```

## Quick Start

```bash
# Initialize profiles from existing settings
ccp init

# Interactive profile selector (default)
ccp

# List all profiles
ccp list

# Switch profile
ccp use <profile-name>

# Show current profile
ccp current
```

## Commands

### Profile Management

| Command | Description |
|---------|-------------|
| `ccp` | Interactive profile selector with arrow keys |
| `ccp list` | List all available profiles |
| `ccp current` | Show current active profile |
| `ccp use <name>` | Switch to a profile |
| `ccp create <name>` | Create new profile from current settings |
| `ccp create <name> --from <other>` | Create profile by copying another |
| `ccp delete <name>` | Delete a profile |
| `ccp copy <src> <dst>` | Copy a profile |
| `ccp rename <old> <new>` | Rename a profile |

### Configuration

| Command | Description |
|---------|-------------|
| `ccp configure` | Interactive configuration of current profile |
| `ccp configure <name>` | Interactive configuration of specific profile |
| `ccp set <key> <value>` | Set a configuration value |
| `ccp set <key> <value> -p <profile>` | Set value in specific profile |
| `ccp get <key>` | Get a configuration value |
| `ccp unset <key>` | Remove a configuration value |

### Import/Export

| Command | Description |
|---------|-------------|
| `ccp export` | Export current profile to stdout |
| `ccp export <name>` | Export specific profile to stdout |
| `ccp import <name>` | Import profile from stdin |
| `ccp diff <p1> <p2>` | Compare two profiles |

### Backup/Restore

| Command | Description |
|---------|-------------|
| `ccp backup` | Create timestamped backup |
| `ccp backup <name>` | Create named backup |
| `ccp restore <name>` | Restore from backup |

### Shell Completions

```bash
# Zsh
ccp completions zsh > ~/.zfunc/_ccp
# or add to .zshrc:
eval "$(ccp completions zsh)"

# Bash
ccp completions bash > /etc/bash_completion.d/ccp

# Fish
ccp completions fish > ~/.config/fish/completions/ccp.fish
```

## File Structure

```
~/.claude/
├── settings.json           # Active config (used by Claude Code)
├── profiles/
│   ├── .current            # Current profile name
│   ├── default.json        # Default profile
│   ├── work.json           # Work profile
│   └── ...
└── backups/
    └── backup-YYYYMMDD-HHMMSS.json
```

## Examples

### Create profiles for different APIs

```bash
# Initialize with current settings
ccp init

# Create profile for z.ai
ccp create z-ai
ccp set env.ANTHROPIC_BASE_URL "https://api.z.ai/api/anthropic" -p z-ai
ccp set env.ANTHROPIC_AUTH_TOKEN "your-token" -p z-ai
ccp set model "opus" -p z-ai

# Create profile for MiniMax
ccp create mini-max
ccp set env.ANTHROPIC_BASE_URL "https://api.minimax.io/anthropic" -p mini-max
ccp set env.ANTHROPIC_AUTH_TOKEN "your-minimax-key" -p mini-max
ccp set env.ANTHROPIC_MODEL "MiniMax-M2.1" -p mini-max
ccp set env.ANTHROPIC_SMALL_FAST_MODEL "MiniMax-M2.1" -p mini-max

# Switch between them
ccp use z-ai
ccp use mini-max
# or just run `ccp` for interactive selection
```

### Backup before experimenting

```bash
ccp backup
# ... make changes ...
ccp restore backup-20260102-120000
```

### Share profiles

```bash
# Export
ccp export work > work-profile.json

# Import on another machine
ccp import work < work-profile.json
```

## How It Works

1. **Profiles** are stored as JSON files in `~/.claude/profiles/`
2. **Switching** (`ccp use`) copies the profile content to `~/.claude/settings.json`
3. **Current profile** name is tracked in `~/.claude/profiles/.current`
4. **Changes** via `ccp set` to the current profile are automatically applied to `settings.json`

## License

MIT
