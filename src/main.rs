use std::{collections::HashMap, io::Write, os::unix::prelude::CommandExt, path::PathBuf, process};

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
    Save {
        /// Bind alias to current folder
        #[clap(long, short('l'))]
        local: bool,
        alias: Alias,
        command: Vec<String>,
    },

    /// Rename an alias
    #[clap(alias("mv"))]
    Rename { source: Alias, destination: Alias },

    /// List aliases
    #[clap(alias("ls"))]
    List {
        /// Show all aliases, even those bound to a different folder
        #[clap(long, short('a'))]
        all: bool,
    },

    /// Edit a command in your editor
    Edit {
        /// Bind alias to current folder
        #[clap(long, short('l'))]
        local: bool,
        alias: Alias,
    },

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
        Action::Save {
            local,
            alias,
            command,
        } => registry.set(alias, Some(command.join(" ")), local),
        Action::Rename {
            source,
            destination,
        } => registry.rename(source, destination),
        Action::Edit { local, alias } => registry.edit(alias, local),
        Action::Delete { alias } => registry.set(alias, None, false),
        Action::List { all } => registry.list(all),
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
    /// List of all aliases
    items: HashMap<Alias, Command>,

    /// Locations where aliases are bound.
    locals: HashMap<Alias, PathBuf>,
}

const STORAGE: &'static str = "ag_registry";
const LOCALS: &'static str = "ag_locals";

impl Registry {
    fn load() -> Result<Self> {
        let items = confy::load(STORAGE).context("Error loading alias registry")?;
        let locals = confy::load(LOCALS).context("Error loading locals registry")?;
        Ok(Registry { items, locals })
    }

    fn set(mut self, alias: Alias, command: Option<Command>, local: bool) -> Result<()> {
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
                self.items.insert(alias.clone(), command);
            }
            None => {
                self.items.remove(&alias);
            }
        }

        // Check bound directory value
        let old_dir = self.locals.get(&alias);
        let new_dir = if local {
            std::env::current_dir().unwrap_or_default()
        } else {
            PathBuf::default()
        };
        if old_dir != Some(&new_dir) {
            println!("old dir: {:?}", old_dir);
            println!("new dir: {:?}", new_dir);
            self.locals.insert(alias, new_dir);
        }

        confy::store(STORAGE, self.items)?;
        confy::store(LOCALS, self.locals)?;
        Ok(())
    }

    /// - Create a temp file and dump the alias content to it
    /// - Allow the user to modify it with $EDITOR
    /// - Save the result and remove the temp file
    fn edit(self, alias: Alias, local: bool) -> Result<()> {
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
        self.set(alias, Some(new_command), local)
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

    fn list(self, all: bool) -> Result<()> {
        let mut aliases: Vec<_> = self
            .items
            .iter()
            .map(|(alias, cmd)| {
                let dir = self
                    .locals
                    .get(alias)
                    .map(Clone::clone)
                    .unwrap_or(PathBuf::new());
                (dir, alias, cmd)
            })
            .collect();
        aliases.sort();
        let current_dir = std::env::current_dir().context("getting current dir")?;
        for (dir, alias, command) in aliases {
            let should_print = all || current_dir.starts_with(&dir);
            if !should_print {
                continue;
            }
            let dir_string = if dir == PathBuf::new() {
                String::new()
            } else {
                format!("[{}] ", dir.to_string_lossy())
            };
            println!("{}{}: {}", dir_string, alias.bright_white(), command);
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
