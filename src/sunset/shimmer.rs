use path_absolutize;
use path_absolutize::Absolutize;
use std::path::PathBuf;
use std::{env, fs};
use std::{path, process};

use pathsearch::find_executable_in_path;

use toml;

pub fn shim(
    target_path: &String,
    args: &Vec<String>,
    shim_name: &Option<String>,
    win: &Option<bool>,
) {
    let target_pathbuf = PathBuf::from(target_path);

    let target = match target_pathbuf.parent() {
        Some(v) => {
            if v == path::Path::new("") {
                match find_executable_in_path(target_path) {
                    Some(path) => path,
                    None => {
                        println!("Cannot find executable in PATH {}", target_path);
                        process::exit(-1);
                    }
                }
            } else {
                println!("Parent {}", v.to_str().unwrap());
                target_pathbuf
                    .to_path_buf()
                    .absolutize()
                    .unwrap()
                    .to_path_buf()
            }
        }
        None => {
            println!("Invalid path {}", target_path);
            process::exit(-1);
        }
    };

    println!("Target {}", target.to_str().unwrap());

    let shim_fullname = match shim_name {
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

    let sunset_dir = get_sunset_dir();
    let shims_dir = get_shims_dir();
    let shim_exe = get_shim_exe(&sunset_dir, win.as_ref().unwrap());
    let shimfile_path = get_shimfile(&shims_dir, &shim_fullname);
    let shimmed_exe_path = get_shimmed_exe(&shims_dir, &shim_fullname);

    println!(
        "Shimming {:?}/ {:?} => {:?} using {:?}",
        shimmed_exe_path, shimfile_path, target, shim_exe
    );

    let mut shimfile_content = toml::value::Table::new();

    let path_value = toml::Value::from(target.to_str().unwrap());

    let argsvec = toml::Value::from(args.to_vec());

    shimfile_content.insert(String::from("path"), path_value);
    shimfile_content.insert(String::from("args"), toml::Value::from(argsvec));

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

    shim_create(&shim_exe, &shimmed_exe_path);

    println!("Done");
}

pub fn shim_path(name: &Option<String>) {
    let name = match name {
        None => {
            println!("{}", get_shims_dir().to_str().unwrap());
            process::exit(0);
        }
        Some(value) => value,
    };

    // Print path of shimfile if exists
    let shimfile_path = get_shimfile(&get_shims_dir(), name);
    println!("{}", shimfile_path.to_str().unwrap());
}

pub fn shim_edit(name: &String) {
    // Print path of shimfile if exists
    let shimfile_path = get_shimfile(&get_shims_dir(), name);

    open::that(&shimfile_path).expect("Error opening shimfile");
}

pub fn shim_show(name: &String) {
    // Print path of shimfile if exists
    let shimfile_path = get_shimfile(&get_shims_dir(), name);

    let content = match std::fs::read_to_string(&shimfile_path) {
        Ok(content) => content,
        Err(err) => {
            println!("Error reading shimfile: {:?}: {:?}", &shimfile_path, err);
            process::exit(-1);
        }
    };

    println!("{}", content);
}

pub fn get_sunset_dir() -> PathBuf {
    return PathBuf::from(env::current_exe().unwrap().parent().unwrap());
}

pub fn get_shims_dir() -> PathBuf {
    let shims_path = env::var("SUNSET_SHIMS_PATH");

    let shims_path = match shims_path {
        Err(err) => {
            println!("SUNSET_SHIMS_PATH env var not specified: {}", err);
            process::exit(-1);
        }
        Ok(value) => value,
    };

    return PathBuf::from(shims_path);
}

pub fn get_shim_exe(sunset_dir: &PathBuf, win: &bool) -> PathBuf {
    let current_exe_base = if *win { "shimw.exe" } else { "shim.exe" };

    let current_exe: PathBuf = [&sunset_dir, &PathBuf::from(current_exe_base)]
        .iter()
        .collect();
    return current_exe;
}

pub fn get_shimmed_exe(shims_dir: &PathBuf, name: &String) -> PathBuf {
    let exefile_basename = PathBuf::from(String::from(name) + ".exe");
    let exefile_path: PathBuf = [&shims_dir, &exefile_basename].iter().collect();
    return exefile_path;
}

pub fn get_shimfile(shims_dir: &PathBuf, name: &String) -> PathBuf {
    let shimfile_basename = PathBuf::from(String::from(name) + ".shim");
    let shimfile_path: PathBuf = [&shims_dir, &shimfile_basename].iter().collect();
    return shimfile_path;
}

pub fn shim_remove(name: &Option<String>) {
    let name = match name {
        None => {
            println!("shim name not specified");
            process::exit(-1);
        }
        Some(value) => value,
    };

    let shims_dir = get_shims_dir();
    // Print path of shimfile if exists
    let shimfile_path = get_shimfile(&shims_dir, name);
    let shimmed_exe_path = get_shimmed_exe(&shims_dir, name);

    println!(
        "Removing shim {:?} ({:?}, {:?})",
        name, shimmed_exe_path, shimfile_path
    );

    if shimfile_path.exists() {
        fs::remove_file(shimfile_path).expect("Not deleted shimfile");
    }

    if shimmed_exe_path.exists() {
        fs::remove_file(shimmed_exe_path).expect("Not deleted shimmed exe path");
    }
}

pub fn shim_create(shim_exe: &PathBuf, shimmed_exe_path: &PathBuf) {
    if shimmed_exe_path.is_file() || shimmed_exe_path.is_symlink() {
        println!("Removing {:?}", &shimmed_exe_path);

        match fs::remove_file(&shimmed_exe_path) {
            Ok(_) => {}
            Err(err) => {
                println!("Cannot remove {:?}: {}", &shimmed_exe_path, err);
                process::exit(-1);
            }
        }
    }

    println!("Creating: {:?}", &shimmed_exe_path);
    match fs::hard_link(&shim_exe, &shimmed_exe_path) {
        Ok(_) => {}
        Err(err) => {
            println!(
                "Cannot hard-link {:?} to {:?}: {:?}",
                &shim_exe, &shimmed_exe_path, err
            );
            process::exit(-1);
        }
    }
}

pub fn shim_list(shim_dir: &PathBuf) -> Vec<String> {
    let files = fs::read_dir(shim_dir).unwrap();

    return files
        .map(|it| it.unwrap().path())
        .filter(|it| str::ends_with(it.file_name().unwrap().to_str().unwrap(), ".shim"))
        .map(|it| String::from(it.file_stem().unwrap().to_str().unwrap()))
        .collect::<Vec<String>>();
}
