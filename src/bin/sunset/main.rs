use clap::{ArgAction, Parser, Subcommand};
use std::process;

use sunset::shim;
use sunset::shimmer;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Adds files to myapp
    Shim {
        #[clap(long, action=ArgAction::SetTrue)]
        win: Option<bool>,

        #[clap(value_parser)]
        path: Option<String>,

        #[clap(value_parser)]
        name: Option<String>,
    },

    Path {
        #[clap(value_parser)]
        name: Option<String>,
    },

    Rm {
        #[clap(value_parser)]
        name: Option<String>,
    },

    Upgrade {
        #[clap(value_parser)]
        name: Option<String>,
    },

    List {},

    UpgradeAll {},
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Shim {
            win,
            path: target_path,
            name,
        } => shimmer::shim(target_path, name, win),
        Commands::Path { name } => shimmer::shim_path(name),
        Commands::Rm { name } => shimmer::shim_remove(name),
        Commands::Upgrade { name } => shim_upgrade(name),
        Commands::List {} => shim_list(),
        Commands::UpgradeAll {} => shim_upgrade_all(),
    };
}

fn shim_upgrade(shim_name: &Option<String>) {
    let shim_name = match shim_name {
        None => {
            println!("shim name not specified");
            process::exit(-1);
        }
        Some(value) => value,
    };

    let sunset_dir = shimmer::get_sunset_dir();
    let shims_dir = shimmer::get_shims_dir();

    let shim_path = shimmer::get_shimfile(&shims_dir, &shim_name);
    let shimmed_exe_path = shimmer::get_shimmed_exe(&shims_dir, &shim_name);

    if !shim_path.exists() {
        println!("Shim {:?} doesn't exists", shim_name);
        process::exit(-1);
    }

    let config = shim::read_config(shim_path.as_path()).expect("Error reading file");
    let shim_exe = shimmer::get_shim_exe(&sunset_dir, &config.win);

    println!("Upgrading {:?} with {:?}", shimmed_exe_path, shim_exe);

    shimmer::shim_create(&shim_exe, &shimmed_exe_path);
}

fn shim_list() {
    let shim_dir = shimmer::get_shims_dir();
    let shim_list = shimmer::shim_list(&shim_dir);

    for shim in shim_list {
        println!("{}", &shim);
    }
}

fn shim_upgrade_all() {
    let sunset_dir = shimmer::get_sunset_dir();
    let shim_dir = shimmer::get_shims_dir();
    let shim_list = shimmer::shim_list(&shim_dir);

    for it in shim_list {
        let shim_path = shimmer::get_shimfile(&shim_dir, &it);
        let shimmed_exe_path = shimmer::get_shimmed_exe(&shim_dir, &it);

        println!("Upgrading shim for {:?}", shim_path);

        let config = shim::read_config(shim_path.as_path()).expect("Error reading file");
        let shim_exe = shimmer::get_shim_exe(&sunset_dir, &config.win);

        println!("Upgrading {:?} with {:?}", shimmed_exe_path, shim_exe);

        shimmer::shim_create(&shim_exe, &shimmed_exe_path);
    }
}
