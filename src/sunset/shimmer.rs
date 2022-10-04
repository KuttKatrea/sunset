use path_absolutize;
use path_absolutize::Absolutize;
use std::path::PathBuf;
use std::process;
use std::{env, fs};

use toml;

pub fn shim(target_path: &Option<String>, name: &Option<String>, win: &Option<bool>) {
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

    let sunset_dir = get_sunset_dir();
    let shims_dir = get_shims_dir();
    let shim_exe = get_shim_exe(&sunset_dir, win.as_ref().unwrap());
    let shimfile_path = get_shimfile(&shims_dir, &shim_name);
    let shimmed_exe_path = get_shimmed_exe(&shims_dir, &shim_name);

    println!(
        "Shimming {:?} => {:?} using {:?}",
        shimmed_exe_path, shimfile_path, shim_exe
    );

    let mut shimfile_content = toml::value::Table::new();

    let path_value = toml::Value::from(target.to_str().unwrap());

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

    shim_create(&shim_exe, &shimmed_exe_path);

    println!("Done");
}

pub fn shim_path(name: &Option<String>) {
    let name = match name {
        None => {
            println!("shim name not specified");
            process::exit(-1);
        }
        Some(value) => value,
    };

    // Print path of shimfile if exists
    let shimfile_path = get_shimfile(&get_shims_dir(), name);
    println!("{}", shimfile_path.to_str().unwrap());
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
