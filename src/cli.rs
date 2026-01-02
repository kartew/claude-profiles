use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(name = "ccp")]
#[command(author, version, about = "Claude Code Profiles - manage your Claude Code settings")]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Interactive profile selector (default when no command given)
    #[command(hide = true)]
    Interactive,
    
    /// List all available profiles
    List,
    
    /// Show current active profile
    Current,
    
    /// Switch to a profile
    Use {
        /// Profile name to switch to
        name: String,
    },
    
    /// Create a new profile
    Create {
        /// Name for the new profile
        name: String,
        /// Copy settings from existing profile
        #[arg(short, long)]
        from: Option<String>,
    },
    
    /// Delete a profile
    Delete {
        /// Profile name to delete
        name: String,
        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },
    
    /// Copy a profile
    Copy {
        /// Source profile
        src: String,
        /// Destination profile name
        dst: String,
    },
    
    /// Rename a profile
    Rename {
        /// Current profile name
        old: String,
        /// New profile name
        new: String,
    },
    
    /// Interactive configuration
    Configure {
        /// Profile to configure (default: current)
        #[arg(short, long)]
        profile: Option<String>,
        /// Profile name (positional, same as --profile)
        #[arg(value_name = "PROFILE")]
        name: Option<String>,
    },
    
    /// Set a configuration value
    Set {
        /// Key path (e.g., "model" or "env.ANTHROPIC_BASE_URL")
        key: String,
        /// Value to set
        value: String,
        /// Profile to modify (default: current)
        #[arg(short, long)]
        profile: Option<String>,
    },
    
    /// Get a configuration value
    Get {
        /// Key path (e.g., "model" or "env.ANTHROPIC_BASE_URL")
        key: String,
        /// Profile to read from (default: current)
        #[arg(short, long)]
        profile: Option<String>,
    },
    
    /// Unset/remove a configuration value
    Unset {
        /// Key path to remove
        key: String,
        /// Profile to modify (default: current)
        #[arg(short, long)]
        profile: Option<String>,
    },
    
    /// Export profile to stdout as JSON
    Export {
        /// Profile to export (default: current)
        name: Option<String>,
    },
    
    /// Import profile from stdin
    Import {
        /// Name for the imported profile
        name: String,
    },
    
    /// Compare two profiles
    Diff {
        /// First profile
        profile1: String,
        /// Second profile
        profile2: String,
    },
    
    /// Create a backup of current settings
    Backup {
        /// Custom backup name
        name: Option<String>,
    },
    
    /// Restore from a backup
    Restore {
        /// Backup name to restore
        backup: String,
    },
    
    /// Initialize profiles directory structure
    Init,
    
    /// Generate shell completions
    Completions {
        /// Shell type
        #[arg(value_enum)]
        shell: Shell,
    },
}
