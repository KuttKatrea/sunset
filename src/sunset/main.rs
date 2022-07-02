use clap::{Parser, Subcommand};
use path_absolutize;
use path_absolutize::Absolutize;
use std::path::PathBuf;
use std::process;
use std::{env, fs};

use toml;

use std::os;

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
        } => shimmer(target_path, name),
        Commands::Path { name } => path(name),
    };
}

fn shimmer(target_path: &Option<String>, name: &Option<String>) {
    let target_path = match target_path {
        None => {
            println!("target path not specified");
            process::exit(-1);
        }
        Some(value) => value,
    };

    let target_pathbuf = PathBuf::from(target_path);

    let target = match target_pathbuf.absolutize() {
        Ok(absolute_path) => absolute_path,
        Err(err) => {
            println!("Cannot absolutize path {}: {}", target_path, err);
            process::exit(-1);
        }
    };

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

    let (_current_exe, shimfile_path, _exefile_path) = get_shimfile(&shim_name);

    println!("Shimming {:?} => {:?}", _exefile_path, &shimfile_path);

    let mut shimfile_content = toml::value::Table::new();

    let path_value = toml::Value::from(target.to_str().expect(""));

    shimfile_content.insert(String::from("path"), path_value);

    let toml_content = match toml::to_string(&shimfile_content) {
        Ok(text_content) => text_content,
        Err(err) => {
            println!("Cannot create TOML string: {:?}", err);
            process::exit(-1);
        }
    };

    match fs::write(&shimfile_path, &toml_content) {
        Ok(_) => {}
        Err(err) => {
            println!("Cannot write file {:?}: {}", shimfile_path, err);
            process::exit(-1);
        }
    }

    if _exefile_path.exists() {
        match fs::remove_file(&_exefile_path) {
            Ok(_) => {}
            Err(err) => {
                println!("Cannot remove {:?}: {}", &_exefile_path, err);
                process::exit(-1);
            }
        }
    }

    match os::windows::fs::symlink_file(&_current_exe, &_exefile_path) {
        Ok(_) => {}
        Err(err) => {
            println!(
                "Cannot symlink {:?} to {:?}: {:?}",
                &_current_exe, &_exefile_path, err
            );
            process::exit(-1);
        }
    }

    println!("Done");
}

fn path(name: &Option<String>) {
    let name = match name {
        None => {
            println!("shim name not specified");
            process::exit(-1);
        }
        Some(value) => value,
    };

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
