# moltctrl

Security-hardened OpenClaw AI agent instance manager. Deploy, manage, and chat with isolated AI agent instances from a single binary.

## Install

### Linux / macOS

```bash
curl -fsSL https://raw.githubusercontent.com/takschdube/moltctrl/main/install.sh | sh
```

With a specific version:

```bash
curl -fsSL https://raw.githubusercontent.com/takschdube/moltctrl/main/install.sh | sh -s -- v0.2.0
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/takschdube/moltctrl/main/install.ps1 | iex
```

Or manually: download the `.zip` for your platform from the [releases page](https://github.com/takschdube/moltctrl/releases), extract, and add to your PATH.

## Uninstall

### Linux / macOS

```bash
curl -fsSL https://raw.githubusercontent.com/takschdube/moltctrl/main/install.sh | sh -s -- --uninstall
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/takschdube/moltctrl/main/install.ps1 | iex -Uninstall
```

Or manually:

```bash
# Linux/macOS
sudo rm /usr/local/bin/moltctrl
rm -rf ~/.moltctrl

# Windows — remove moltctrl.exe from your PATH directory and delete %USERPROFILE%\.moltctrl
```

## Quick Start

```bash
# Check system requirements
moltctrl doctor

# Create an instance
moltctrl create myagent --provider anthropic --api-key sk-ant-...

# Chat with it
moltctrl chat myagent

# Clean up
moltctrl destroy myagent --force
```

## Features

- **Single binary** — install in seconds, no runtime dependencies beyond Docker
- **Built-in WebSocket chat** — no external tools needed
- **Provider-agnostic** — Anthropic, OpenAI, Google, AWS Bedrock, OpenRouter, Ollama
- **Cross-platform** — Linux, macOS, and Windows
- **Dual isolation** — Docker containers (15 security hardening measures) or process sandbox
- **Auto port allocation** — finds available ports in the 18789-18889 range
- **Pairing key management** — approve, list, and revoke access keys

## Commands

| Command | Description |
|---------|-------------|
| `create <name>` | Create and start a new instance |
| `destroy <name>` | Stop and remove an instance and its data |
| `list` | List all instances |
| `status <name>` | Show detailed instance status |
| `start <name>` | Start a stopped instance |
| `stop <name>` | Stop a running instance |
| `restart <name>` | Restart an instance |
| `logs <name>` | View instance logs |
| `token <name>` | Show or regenerate auth token |
| `open <name>` | Open instance in browser |
| `pair approve <name>` | Create a new pairing key |
| `pair list <name>` | List all pairing keys |
| `pair revoke <name>` | Revoke a pairing key |
| `update <name>` | Update instance configuration |
| `chat <name>` | Interactive WebSocket chat |
| `doctor` | Check system requirements |

## Command Reference

### create

```bash
moltctrl create <name> [options]
```

| Flag | Description | Default |
|------|-------------|---------|
| `--provider` | AI provider | interactive prompt |
| `--api-key` | API key | env var or prompt |
| `--model` | Model name | provider-specific |
| `--port` | Host port | auto (18789-18889) |
| `--image` | Docker image | `ghcr.io/openclaw/openclaw:latest` |
| `--mem` | Memory limit | `2g` |
| `--cpus` | CPU limit | `2` |
| `--pids` | PID limit | `256` |
| `--docker` | Use Docker isolation | default |
| `--process` | Use process sandbox | |

### destroy

```bash
moltctrl destroy <name> [--force]
```

### logs

```bash
moltctrl logs <name> [--follow] [--tail N]
```

### token

```bash
moltctrl token <name> [--regenerate]
```

### update

```bash
moltctrl update <name> [--model MODEL] [--mem MEM] [--cpus CPUS] [--pids PIDS]
```

### pair

```bash
moltctrl pair approve <name> [--label LABEL]
moltctrl pair list <name>
moltctrl pair revoke <name> --label LABEL
```

## Isolation Modes

### Docker Mode (default)

15 security hardening measures:
- Non-root user (1000:1000)
- Read-only root filesystem
- All capabilities dropped (only NET_BIND_SERVICE, DAC_OVERRIDE added)
- `no-new-privileges` security option
- Memory, CPU, and PID limits
- localhost-only port binding (127.0.0.1)
- Named volumes only (no bind mounts)
- Log rotation (10MB, 3 files)
- Health checks every 30s

### Process Sandbox Mode

Lightweight isolation using OS-level resource limits:
- **Linux/macOS**: `ulimit` via the `nix` crate (RLIMIT_AS, RLIMIT_NPROC, RLIMIT_CPU, RLIMIT_FSIZE, RLIMIT_CORE)
- **Windows**: Job Objects via `windows-sys`

Use `--process` flag with `moltctrl create` to use process mode instead of Docker.

## Provider Configuration

| Provider | Env Variable | Default Model |
|----------|-------------|---------------|
| `anthropic` | `ANTHROPIC_API_KEY` | `claude-sonnet-4-20250514` |
| `openai` | `OPENAI_API_KEY` | `gpt-4o` |
| `google` | `GOOGLE_API_KEY` | `gemini-2.0-flash` |
| `aws-bedrock` | `AWS_ACCESS_KEY_ID` | `anthropic.claude-sonnet-4-20250514-v1:0` |
| `openrouter` | `OPENROUTER_API_KEY` | `anthropic/claude-sonnet-4-20250514` |
| `ollama` | *(none)* | `llama3.1` |

Providers can be set via `--provider` flag, `MOLTCTRL_PROVIDER` env var, or interactive prompt.

## Architecture

```
~/.moltctrl/
└── instances/
    └── myagent/
        ├── instance.json      # Instance state and metadata
        ├── .env               # Provider credentials (mode 600)
        └── docker-compose.yml # Generated Docker Compose config
```

## Building from Source

```bash
git clone https://github.com/takschdube/moltctrl.git
cd moltctrl
cargo build --release
sudo cp target/release/moltctrl /usr/local/bin/
```

## Global Options

| Flag | Description |
|------|-------------|
| `-v, --verbose` | Enable debug output |
| `--no-color` | Disable colored output |
| `--force` | Skip confirmation prompts |

## License

MIT
