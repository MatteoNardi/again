use std::{collections::HashMap, os::unix::prelude::CommandExt};

use anyhow::{Context, Result};

type Alias = String;
type Command = String;

use clap::{IntoApp, Parser, Subcommand};
use colored::Colorize;

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
    /// Edit a command in your editor
    Edit { alias: Alias },
    /// Generate again shell completions for your shell to stdout
    Completions {
        /// How you want to invoke again
        #[clap(long, default_value = "again")]
        exe: String,
        #[clap(arg_enum)]
        shell: clap_complete::Shell,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    let registry = Registry::load()?;
    match args.action {
        Action::Save { alias, command } => registry.set(alias, Some(command.join(" "))),
        Action::Edit { alias } => registry.edit(alias),
        Action::Delete { alias } => registry.set(alias, None),
        Action::List => registry.list(),
        Action::Run { alias } => registry.run(alias),
        Action::Completions { exe, shell } => {
            clap_complete::generate(
                shell,
                &mut Args::command(),
                exe,
                &mut std::io::stdout().lock(),
            );
            Ok(())
        }
    }
}

struct Registry {
    items: HashMap<Alias, Command>,
}

const STORAGE: &'static str = "ag_registry";

impl Registry {
    fn load() -> Result<Self> {
        let items = confy::load(STORAGE).context("Error loading alias registry")?;
        Ok(Registry { items })
    }

    fn set(mut self, alias: Alias, command: Option<Command>) -> Result<()> {
        println!("{}:", alias.bright_white());
        // Delete old value
        if let Some(old_value) = self.items.get(&alias) {
            println!("  {}", old_value.strikethrough());
        }
        // Set the new value
        match command {
            Some(command) => {
                println!("  {}", command);
                self.items.insert(alias, command);
            }
            None => {
                self.items.remove(&alias);
            }
        }
        confy::store(STORAGE, self.items)?;
        Ok(())
    }

    fn edit(self, alias: Alias) -> Result<()> {
        //    // TODO: support alias editing:
        //    // - Create a temp file and dump the alias content to it
        //    // - Allow the user to modify it with $EDITOR
        //    // - Save the result and remove the temp file
        //fn edit(value: &str) -> Result<String> {
        //}
        todo!()
    }

    fn list(self) -> Result<()> {
        for (alias, command) in self.items {
            println!("{}: {}", alias.bright_white(), command);
        }
        Ok(())
    }

    fn run(self, alias: Alias) -> Result<()> {
        match self.items.get(&alias) {
            Some(command) => {
                println!("{}: {}", alias.bright_white(), command);
                let error = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .exec();
                println!("Error running command {:?}", error)
            }
            None => println!("Alias not found: {}", alias),
        }
        Ok(())
    }
}
