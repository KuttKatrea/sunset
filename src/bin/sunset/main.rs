use clap::{Parser, Subcommand, ArgAction};
use std::env;
use std::fs;
use std::path;
use std::process;

use winreg::enums::HKEY_CURRENT_USER;
use winreg::enums::KEY_ALL_ACCESS;
use winreg::RegKey;

use sunset::shim;
use sunset::shimmer;

/// Create shims to executables with default arguments and environment in Windows.
#[derive(Parser)]
#[clap(trailing_var_arg = true)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(long, env = "SUNSET_SHIMS_PATH")]
        shims_path: Option<String>,
    },

    /// Adds files to myapp
    Shim {
        #[arg(long)]
        shim_name: Option<String>,

        #[arg(long, action=ArgAction::SetTrue)]
        win: Option<bool>,

        #[arg(long, action=ArgAction::SetTrue)]
        hidden: Option<bool>,

        #[arg(long, action=ArgAction::SetFalse)]
        no_wait: Option<bool>,

        #[arg(value_parser)]
        path: String,

        #[arg(value_parser, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    Path {
        #[arg(value_parser)]
        name: Option<String>,
    },

    Edit {
        #[arg(value_parser)]
        name: String,
    },

    Show {
        #[arg(value_parser, name = "SHIM NAME")]
        name: String,
    },

    Rm {
        #[arg(value_parser)]
        name: Option<String>,
    },

    Upgrade {
        #[arg(value_parser)]
        name: Option<String>,
    },

    List {},

    UpgradeAll {},
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { shims_path } => shim_init(shims_path),
        Commands::Shim {
            shim_name,
            win,
            hidden,
            no_wait,
            path: target_path,
            args,
        } => shimmer::shim(target_path, args, shim_name, win, hidden),
        Commands::Path { name } => shimmer::shim_path(name),
        Commands::Edit { name } => shimmer::shim_edit(name),
        Commands::Show { name } => shimmer::shim_show(name),
        Commands::Rm { name } => shimmer::shim_remove(name),
        Commands::Upgrade { name } => shim_upgrade(name),
        Commands::List {} => shim_list(),
        Commands::UpgradeAll {} => shim_upgrade_all(),
    };
}

fn shim_init(shims_path: &Option<String>) {
    let localappdata_path = match env::var("LOCALAPPDATA") {
        Ok(var_value) => path::Path::new(&var_value).join("sunset\\shims"),
        Err(e) => {
            println!("Failed to get value of {}: {}", "LOCALAPPDATA", e);
            process::exit(-1);
        }
    };

    let selected_shims_path = match shims_path {
        Some(value) => path::Path::new(value),
        None => localappdata_path.as_path(),
    };

    let selected_shims_path_str = selected_shims_path.to_str().unwrap();

    println!("Selected shims path: {:?}", selected_shims_path);

    println!(
        "Initializing sunset, shims in path {:?}",
        selected_shims_path
    );

    // ENSURE SHIMS_PATH exists

    match fs::create_dir_all(selected_shims_path) {
        Ok(_value) => true,
        Err(e) => {
            println!("Error creating directories: {}", e);
            process::exit(-1);
        }
    };

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env_key = hkcu
        .open_subkey_with_flags("Environment", KEY_ALL_ACCESS)
        .unwrap();

    println!(
        "Setting SUNSET_SHIMS_PATH environment variable to {}",
        selected_shims_path_str
    );

    match env_key.set_value("SUNSET_SHIMS_PATH", &selected_shims_path_str) {
        Ok(_) => {}
        Err(err) => {
            println!("Error setting env var: {}", err);
            process::exit(-1);
        }
    };

    // Get the current value of the PATH variable
    let current_path: String = env_key.get_value("PATH").unwrap();

    let paths: Vec<&str> = current_path.split(";").collect();

    let is_present = paths.iter().any(|&part| part == selected_shims_path_str);

    if !is_present {
        // Append your directory to the current PATH
        let new_path = format!("{};{}", current_path, selected_shims_path_str);

        println!("Setting PATH environment variable to: {}", new_path);

        // Update the PATH value in the registry
        env_key.set_value("PATH", &new_path).unwrap();
    } else {
        println!("{} already on PATH", selected_shims_path_str);
    }

    println!("Restart processes");
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
