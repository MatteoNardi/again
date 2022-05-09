use std::{collections::HashMap, io::Write, os::unix::prelude::CommandExt, process};

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

    /// Rename an alias
    #[clap(alias("mv"))]
    Rename { source: Alias, destination: Alias },

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
        Action::Rename {
            source,
            destination,
        } => registry.rename(source, destination),
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
        // trim command and make sure it's not empty
        let command = command
            .map(|x| {
                let trimmed = x.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            })
            .flatten();
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

    /// - Create a temp file and dump the alias content to it
    /// - Allow the user to modify it with $EDITOR
    /// - Save the result and remove the temp file
    fn edit(self, alias: Alias) -> Result<()> {
        let command = self
            .items
            .get(&alias)
            .map(|x| x.to_string())
            .unwrap_or_default();
        let editor = std::env::var("EDITOR").context("EDITOR variable must be set")?;
        let file = tempfile::NamedTempFile::new()?;
        writeln!(file.as_file(), "{}", command)?;
        process::Command::new(&editor)
            .arg(file.path())
            .status()
            .context("running editor failed")?;
        let new_command = std::fs::read_to_string(file.path())?;
        self.set(alias, Some(new_command))
    }

    fn rename(mut self, source: Alias, destination: Alias) -> Result<()> {
        if !self.items.contains_key(&source) {
            println!("{} doesn't exist", source.bright_white());
            return Ok(());
        }
        if let Some(value) = self.items.get(&destination) {
            println!("{} already exists with value", destination.bright_white());
            println!("  {}", value);
            return Ok(());
        }
        let cmd = self.items.remove(&source).unwrap();
        println!(
            "{} => {}",
            source.strikethrough(),
            destination.bright_white()
        );
        println!("    {}", cmd);
        self.items.insert(destination, cmd);
        confy::store(STORAGE, self.items)?;
        Ok(())
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
                let error = process::Command::new("sh").arg("-c").arg(command).exec();
                println!("Error running command {:?}", error)
            }
            None => println!("Alias not found: {}", alias),
        }
        Ok(())
    }
}
