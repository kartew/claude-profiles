mod cli;
mod config;
mod profile;
mod commands;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let command = cli.command.unwrap_or(Commands::Interactive);
    
    match command {
        Commands::Interactive => commands::interactive()?,
        Commands::List => commands::list()?,
        Commands::Current => commands::current()?,
        Commands::Use { name } => commands::use_profile(&name)?,
        Commands::Create { name, from } => commands::create(&name, from.as_deref())?,
        Commands::Delete { name, force } => commands::delete(&name, force)?,
        Commands::Copy { src, dst } => commands::copy(&src, &dst)?,
        Commands::Rename { old, new } => commands::rename(&old, &new)?,
        Commands::Configure { profile, name } => commands::configure(profile.or(name).as_deref())?,
        Commands::Set { key, value, profile } => commands::set(&key, &value, profile.as_deref())?,
        Commands::Get { key, profile } => commands::get(&key, profile.as_deref())?,
        Commands::Unset { key, profile } => commands::unset(&key, profile.as_deref())?,
        Commands::Export { name } => commands::export(name.as_deref())?,
        Commands::Import { name } => commands::import(&name)?,
        Commands::Diff { profile1, profile2 } => commands::diff(&profile1, &profile2)?,
        Commands::Backup { name } => commands::backup(name.as_deref())?,
        Commands::Restore { backup } => commands::restore(&backup)?,
        Commands::Init => commands::init()?,
        Commands::Completions { shell } => commands::completions(shell)?,
    }
    
    Ok(())
}
