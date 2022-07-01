use clap::{Parser, Subcommand};
use serde::de::Unexpected::Str;
use std::ops::Deref;
use std::path;
use std::path::PathBuf;
use std::{env, fs};
use toml;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Adds files to myapp
    Shimmer {
        #[clap(value_parser)]
        path: Option<String>,

        #[clap(value_parser)]
        name: Option<String>,
    },

    Path {
        #[clap(value_parser)]
        name: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Shimmer {
            path: target_path,
            name,
        } => match target_path {
            None => {}
            Some(target_path) => {
                shimmer(target_path, name);
            }
        },
        Commands::Path { name } => match name {
            None => {}
            Some(name) => {
                path(name);
            }
        },
    }
}

fn shimmer(target_path: &String, name: &Option<String>) {
    let target = PathBuf::from(target_path).canonicalize().expect("");

    let shim_name = match name {
        None => String::from(
            target
                .with_extension("")
                .file_stem()
                .expect("Nope")
                .to_str()
                .expect("Nope"),
        ),
        Some(name) => String::from(name),
    };

    println!("{}", shim_name);

    let (_current_exe, shimfile_path, _exefile_path) = get_shimfile(&shim_name);

    let mut shimfile_content = toml::value::Table::new();
    let path_value = toml::Value::from(target.to_str().expect(""));

    shimfile_content.insert(String::from("path"), path_value);

    let toml_content = toml::to_string(&shimfile_content).expect("");

    let res = std::fs::write(shimfile_path, &toml_content);

    if (_exefile_path.exists()) {
        let res = fs::remove_file(&_exefile_path);
    }

    let res = std::os::windows::fs::symlink_file(_current_exe, _exefile_path);

    // Get own dir
    // If exe name not specified, get targeted exe name
    // Create .shim file with path = targeted_exe
    // LN shim exe to exename.exe
}

fn path(name: &String) {
    // Print path of shimfile if exists
    let (_current_exe, shimfile_path, _exefile_path) = get_shimfile(name);
    println!("{}", shimfile_path.to_str().expect(""));
}

fn get_shimfile(name: &String) -> (PathBuf, PathBuf, PathBuf) {
    let current_dir = PathBuf::from(env::current_exe().expect("").parent().expect(""));
    let shimfile_basename = PathBuf::from(String::from(name) + ".shim");
    let exefile_basename = PathBuf::from(String::from(name) + ".exe");

    let shimfile_path: PathBuf = [&current_dir, &shimfile_basename].iter().collect();
    let exefile_path: PathBuf = [&current_dir, &exefile_basename].iter().collect();
    let current_exe: PathBuf = [&current_dir, &PathBuf::from("shim.exe")].iter().collect();

    return (current_exe, shimfile_path, exefile_path);
}
