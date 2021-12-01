use clap::{crate_authors, crate_name, crate_version, App, AppSettings, Arg, SubCommand};

fn main() {
    let name = Arg::with_name("name").required(true).help("Command alias");
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about("A commands alias manager")
        .setting(AppSettings::SubcommandRequired)
        .subcommand(
            SubCommand::with_name("save")
                .setting(AppSettings::TrailingVarArg)
                .about("Save a command")
                .arg(name.clone())
                .arg(Arg::with_name("command").multiple(true)),
        )
        .subcommand(
            SubCommand::with_name("edit")
                .about("Edit a command in your editor")
                .arg(name.clone()),
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("Run an alias")
                .arg(name.clone()),
        )
        .subcommand(
            SubCommand::with_name("delete")
                .about("Remove a command")
                .arg(name.clone()),
        )
        .get_matches();
    println!("{:?}", matches);
}
