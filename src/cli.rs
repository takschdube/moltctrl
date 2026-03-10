use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "moltctrl",
    about = "Security-hardened OpenClaw AI agent instance manager",
    version,
    after_help = "Run 'moltctrl help <command>' for command-specific help."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable debug output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Skip confirmation prompts
    #[arg(long, global = true)]
    pub force: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create and start a new instance
    Create {
        /// Instance name
        name: String,

        /// AI provider (anthropic, openai, google, aws-bedrock, openrouter, ollama)
        #[arg(long)]
        provider: Option<String>,

        /// API key for the provider
        #[arg(long)]
        api_key: Option<String>,

        /// Model name (default: provider-specific)
        #[arg(long)]
        model: Option<String>,

        /// Host port (default: auto-allocated 18789-18889)
        #[arg(long)]
        port: Option<u16>,

        /// Docker image (default: ghcr.io/openclaw/openclaw:latest)
        #[arg(long)]
        image: Option<String>,

        /// Memory limit (default: 2g)
        #[arg(long)]
        mem: Option<String>,

        /// CPU limit (default: 2)
        #[arg(long)]
        cpus: Option<String>,

        /// PID limit (default: 256)
        #[arg(long)]
        pids: Option<String>,

        /// Use Docker isolation mode
        #[arg(long, conflicts_with = "process")]
        docker: bool,

        /// Use process sandbox isolation mode
        #[arg(long, conflicts_with = "docker")]
        process: bool,
    },

    /// Stop and remove an instance and its data
    Destroy {
        /// Instance name
        name: String,

        /// Skip confirmation prompt
        #[arg(long, short)]
        force: bool,
    },

    /// List all instances
    #[command(alias = "ls")]
    List,

    /// Show detailed instance status
    Status {
        /// Instance name
        name: String,
    },

    /// Start a stopped instance
    Start {
        /// Instance name
        name: String,
    },

    /// Stop a running instance
    Stop {
        /// Instance name
        name: String,
    },

    /// Restart an instance
    Restart {
        /// Instance name
        name: String,
    },

    /// View instance logs
    Logs {
        /// Instance name
        name: String,

        /// Follow log output
        #[arg(long, short)]
        follow: bool,

        /// Number of lines to show (default: 100)
        #[arg(long, default_value = "100")]
        tail: String,
    },

    /// Show or regenerate auth token
    Token {
        /// Instance name
        name: String,

        /// Generate a new token (requires restart to apply)
        #[arg(long)]
        regenerate: bool,
    },

    /// Open instance in browser
    Open {
        /// Instance name
        name: String,
    },

    /// Manage pairing keys
    Pair {
        #[command(subcommand)]
        subcmd: PairCommands,
    },

    /// Update instance configuration
    Update {
        /// Instance name
        name: String,

        /// Change the AI model
        #[arg(long)]
        model: Option<String>,

        /// Change memory limit
        #[arg(long)]
        mem: Option<String>,

        /// Change CPU limit
        #[arg(long)]
        cpus: Option<String>,

        /// Change PID limit
        #[arg(long)]
        pids: Option<String>,
    },

    /// Interactive WebSocket chat
    Chat {
        /// Instance name
        name: String,
    },

    /// Check system requirements
    Doctor,

    /// Show version
    Version,
}

#[derive(Subcommand)]
pub enum PairCommands {
    /// Create a new pairing key
    Approve {
        /// Instance name
        name: String,

        /// Label for the pairing key
        #[arg(long)]
        label: Option<String>,
    },

    /// List all pairing keys
    List {
        /// Instance name
        name: String,
    },

    /// Revoke a pairing key
    Revoke {
        /// Instance name
        name: String,

        /// Label of the key to revoke
        #[arg(long)]
        label: String,
    },
}
