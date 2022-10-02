use std::env;
use std::path::Path;
use std::process;
use std::process::Command;
use toml::value::Table;

#[derive(Debug)]
enum ShimConfigEnvAction {
    SET,
    CLEAR,
    APPEND,
    PREPREND,
}

#[derive(Debug)]
struct ShimConfigEnvActionItem {
    var: String,
    action: ShimConfigEnvAction,
    value: String,
    separator: String,
}

#[derive(Debug)]
struct ShimConfig {
    path: String,
    args: Vec<String>,
    env: Vec<ShimConfigEnvActionItem>,
}

fn map_single_env_action_item(table: &Table) -> ShimConfigEnvActionItem {
    let var = table.get("var").expect("VAR").as_str().unwrap();

    let action_str = match table.get("action") {
        None => "set",
        Some(action) => action.as_str().unwrap(),
    };

    let action = match action_str {
        "append" => ShimConfigEnvAction::APPEND,
        "prepend" => ShimConfigEnvAction::PREPREND,
        "clear" => ShimConfigEnvAction::CLEAR,
        "set" => ShimConfigEnvAction::SET,
        _ => {
            panic!("{} is not a valid action for variable {}", action_str, var)
        }
    };

    let value = match table.get("value") {
        None => "",
        Some(value) => value.as_str().unwrap(),
    };

    let separator = match table.get("separator") {
        None => "",
        Some(separator) => separator.as_str().unwrap(),
    };

    ShimConfigEnvActionItem {
        var: String::from(var),
        action,
        value: String::from(value),
        separator: String::from(separator),
    }
}

fn read_config(path: &Path) -> std::io::Result<ShimConfig> {
    let content = std::fs::read_to_string(path)?;
    let value: Table = toml::from_str(&content).expect("Couldn't get a value");

    let path = String::from(
        value
            .get("path")
            .expect("Doesn't have a path")
            .as_str()
            .unwrap(),
    );

    let args_raw = value.get("args");

    let mut args: Vec<String> = match args_raw {
        None => Vec::new(),
        Some(args_raw) => args_raw
            .as_array()
            .unwrap()
            .iter()
            .map(|e| String::from(e.as_str().unwrap()))
            .collect(),
    };

    let mut cmd_args: Vec<String> = env::args().skip(1).collect();

    args.append(&mut cmd_args);

    let env_raw = value.get("env");

    let env = match env_raw {
        None => Vec::new(),
        Some(env_raw) => env_raw
            .as_array()
            .unwrap()
            .iter()
            .map(|it| map_single_env_action_item(it.as_table().unwrap()))
            .collect(),
    };

    let ret_value = ShimConfig { path, args, env };

    Ok(ret_value)
}

pub fn main() {
    // Catch Signals. If signals, set global semaphore.

    let exe_path = env::current_exe().expect("No arg 0? Crazy");
    //let exe_path = Path::new(&exe_path);
    let shim_path = exe_path.with_extension("shim");

    //println!("Reading exe file at: {:?}", &exe_path);
    //println!("Reading shim file at: {:?}", &shim_path);

    let config = read_config(shim_path.as_path()).expect("Error reading file");

    // dbg!(&config);

    let mut cmd = Command::new(config.path);
    cmd.args(config.args);

    for k in config.env {
        match k.action {
            ShimConfigEnvAction::SET => {
                cmd.env(k.var, k.value);
            }
            ShimConfigEnvAction::CLEAR => {
                cmd.env_remove(k.var);
            }
            ShimConfigEnvAction::APPEND => {
                let current_var_value = match &env::var(&k.var) {
                    Ok(value) => String::from(value),
                    Err(_) => String::from(""),
                };
                let mut value = String::from("");
                value.push_str(&current_var_value);
                value.push_str(&k.separator);
                value.push_str(&k.value);
                cmd.env(k.var, value);
            }
            ShimConfigEnvAction::PREPREND => {
                let current_var_value = match &env::var(&k.var) {
                    Ok(value) => String::from(value),
                    Err(_) => String::from(""),
                };
                let mut value = String::from("");
                value.push_str(&k.value);
                value.push_str(&k.separator);
                value.push_str(&current_var_value);
                cmd.env(k.var, value);
            }
        }
    }

    let mut child = cmd.spawn().expect("SS: Failed to execute command");

    let exit_code = match child.try_wait() {
        Ok(Some(status)) => status.code().unwrap(),
        Ok(None) => {
            let res = child.wait();
            res.unwrap().code().unwrap()
        }
        Err(_e) => -1,
    };

    process::exit(exit_code);
}
