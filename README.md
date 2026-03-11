# moltctrl

Security-hardened AI agent instance manager. Single binary, zero config.

## Install

**Linux / macOS**
```bash
curl -fsSL https://raw.githubusercontent.com/takschdube/moltctrl/main/install.sh | sh
```

**Windows (PowerShell)**
```powershell
irm https://raw.githubusercontent.com/takschdube/moltctrl/main/install.ps1 | iex
```

## Quick Start

Just run it:

```bash
moltctrl
```

The interactive wizard walks you through provider selection, API key entry, and instance creation. No flags needed.

## Headless / Scripting

For CI pipelines and AI agents, use subcommands directly:

```bash
# Create an instance
moltctrl create myagent --provider anthropic --api-key sk-ant-...

# Chat with it
moltctrl chat myagent

# List all instances
moltctrl list

# Clean up
moltctrl destroy myagent --force
```

Use `--json` for machine-readable output. Use `--force` to skip prompts.

## Providers

| Provider | Default Model | Status |
|----------|--------------|--------|
| `anthropic` | `claude-sonnet-4-20250514` | Available |
| `openai` | `gpt-4o` | Available |
| `google` | `gemini-2.0-flash` | Available |
| `openrouter` | `anthropic/claude-sonnet-4-20250514` | Available |
| `aws-bedrock` | `anthropic.claude-sonnet-4-20250514-v1:0` | Coming soon |
| `ollama` | `llama3.1` | Coming soon |

Set via `--provider` flag, `MOLTCTRL_PROVIDER` env var, or interactive prompt. API keys are read from `--api-key`, standard env vars (e.g. `ANTHROPIC_API_KEY`), or prompted interactively.

## Docker Mode

By default, instances run in a lightweight process sandbox. For stronger isolation, use Docker:

```bash
moltctrl create myagent --provider anthropic --api-key sk-ant-... --docker
```

Docker mode applies 15 security hardening measures including read-only rootfs, dropped capabilities, memory/CPU/PID limits, and localhost-only port binding.

## Building from Source

```bash
git clone https://github.com/takschdube/moltctrl.git
cd moltctrl
cargo build --release
sudo cp target/release/moltctrl /usr/local/bin/
```

## License

MIT
