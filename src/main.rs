use std::{collections::HashMap, os::unix::prelude::CommandExt};

use anyhow::{Context, Result};
use clap::{crate_authors, crate_name, crate_version, App, AppSettings, Arg, SubCommand};

type Alias = String;
type Command = String;
type Registry = HashMap<Alias, Command>;

fn main() -> Result<()> {
    let matches = args().get_matches();
    let mut registry: Registry =
        confy::load("ag_registry").context("Error loading alias registry")?;
    match matches.subcommand() {
        ("save", Some(matches)) => {
            let alias = matches.value_of("alias").unwrap();
            let command = matches
                .values_of("command")
                .unwrap()
                .collect::<Vec<&str>>()
                .join(" ");
            let old = registry.insert(alias.to_string(), command);
            if let Some(cmd) = old {
                println!("Replacing old command: {:?}", cmd);
            }
            confy::store("ag_registry", registry)?;
        }
        ("delete", Some(matches)) => {
            let alias = matches.value_of("alias").unwrap();
            match registry.remove(alias) {
                Some(cmd) => println!("Deleted {}:\n{:?}", alias, cmd),
                None => println!("Alias not found: {}", alias),
            }
            confy::store("ag_registry", registry)?;
        }
        ("run", Some(matches)) => {
            let alias = matches.value_of("alias").unwrap();
            match registry.get(alias) {
                Some(command) => {
                    println!("{}: {}", alias, command);
                    let error = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(command)
                        .exec();
                    println!("Error running command {:?}", error)
                }
                None => println!("Alias not found: {}", alias),
            }
        }
        ("ls", _) => {
            for (alias, command) in registry {
                println!("{}: {}", alias, command);
            }
        }
        x => println!("Unexpected data {:?}", x),
    }
    Ok(())
}

fn args() -> App<'static, 'static> {
    let alias = Arg::with_name("alias").required(true).help("Command alias");
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about("A commands alias manager")
        .setting(AppSettings::SubcommandRequired)
        .subcommand(
            SubCommand::with_name("save")
                .setting(AppSettings::TrailingVarArg)
                .about("Save a command")
                .arg(alias.clone())
                .arg(Arg::with_name("command").multiple(true)),
        )
        // TODO: support alias editing:
        // - Create a temp file and dump the alias content to it
        // - Allow the user to modify it with $EDITOR
        // - Save the result and remove the temp file
        // .subcommand(
        //    SubCommand::with_name("edit")
        //        .about("Edit a command in your editor")
        //        .arg(alias.clone()),
        // )
        .subcommand(
            SubCommand::with_name("run")
                .about("Run an alias")
                .arg(alias.clone()),
        )
        .subcommand(
            SubCommand::with_name("delete")
                .about("Remove a command")
                .arg(alias.clone()),
        )
        .subcommand(SubCommand::with_name("ls").about("List aliases"))
}
