# moltctrl

Security-hardened AI agent instance manager.

## Quick Start (for AI agents)

Create and manage OpenClaw agent instances programmatically.

### Create an instance
```bash
moltctrl create <name> --provider anthropic --api-key <key> --json
```

### Chat (single message)
```bash
moltctrl chat <name> --message "Your prompt here" --json
```

### List instances
```bash
moltctrl list --json
```

### Destroy
```bash
moltctrl destroy <name> --force
```

## Providers
- anthropic (default model: claude-sonnet-4-20250514)
- openai (default model: gpt-4o)
- google (default model: gemini-2.0-flash)
- openrouter (default model: anthropic/claude-sonnet-4-20250514)

## Notes
- Use --json flag for machine-readable output
- Use --force to skip confirmation prompts
- Instances persist until explicitly destroyed
- Each instance gets a unique auth token and port
