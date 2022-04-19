use std::{collections::HashMap, os::unix::prelude::CommandExt};

use anyhow::{Context, Result};

type Alias = String;
type Command = String;
type Registry = HashMap<Alias, Command>;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Run an alias
    Run { alias: Alias },

    /// Delete an alias
    #[clap(alias("rm"))]
    Delete { alias: Alias },

    /// Save an alias
    Save { alias: Alias, command: Vec<String> },

    /// List aliases
    #[clap(alias("ls"))]
    List,
    // TODO: support alias editing:
    // - Create a temp file and dump the alias content to it
    // - Allow the user to modify it with $EDITOR
    // - Save the result and remove the temp file
    // /// Edit a command in your editor
    // Edit { alias: Alias },
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut registry: Registry =
        confy::load("ag_registry").context("Error loading alias registry")?;
    match args.action {
        Action::Save { alias, command } => {
            let command = command.join(" ");
            let old = registry.insert(alias.to_string(), command);
            if let Some(alias) = old {
                println!("Replacing old command: {:?}", alias);
            }
            confy::store("ag_registry", registry)?;
        }
        Action::Run { alias } => match registry.get(&alias) {
            Some(command) => {
                println!("{}: {}", alias, command);
                let error = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .exec();
                println!("Error running command {:?}", error)
            }
            None => println!("Alias not found: {}", alias),
        },
        Action::Delete { alias } => {
            match registry.remove(&alias) {
                Some(alias) => println!("Deleted {}:\n{:?}", alias, alias),
                None => println!("Alias not found: {}", alias),
            }
            confy::store("ag_registry", registry)?;
        }
        Action::List => {
            for (alias, command) in registry {
                println!("{}: {}", alias, command);
            }
        }
    }
    Ok(())
}
