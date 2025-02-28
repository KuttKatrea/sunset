use std::env;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process;
use std::process::{Command, Stdio};
use toml::value::Table;
use toml::Value::Boolean;
use regex::{Captures, Regex};
use once_cell::sync::Lazy;

#[derive(Debug)]
enum ShimConfigEnvAction {
    SET,
    CLEAR,
    APPEND,
    PREPREND,
}

#[derive(Debug)]
pub struct ShimConfigEnvActionItem {
    var: String,
    action: ShimConfigEnvAction,
    value: String,
    separator: String,
}

#[derive(Debug)]
pub struct ShimConfig {
    pub path: String,
    pub args: Vec<String>,
    pub env: Vec<ShimConfigEnvActionItem>,
    pub win: bool,
    pub hidden: bool,
    pub wait: bool,
    pub env_expand_path: bool,
    pub env_expand_args: bool,
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

pub fn read_config(path: &Path) -> std::io::Result<ShimConfig> {
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

    let win = value
        .get("win")
        .unwrap_or(&Boolean(false))
        .as_bool()
        .unwrap();

    let hidden = value
        .get("hidden")
        .unwrap_or(&Boolean(false))
        .as_bool()
        .unwrap();

    let wait = value
        .get("wait")
        .unwrap_or(&Boolean(true))
        .as_bool()
        .unwrap();

    let env_expand_path = value
        .get("env_expand_path")
        .unwrap_or(&Boolean(false))
        .as_bool()
        .unwrap();

    let env_expand_args = value
        .get("env_expand_args")
        .unwrap_or(&Boolean(false))
        .as_bool()
        .unwrap();

    let ret_value = ShimConfig {
        path,
        args,
        env,
        win,
        hidden,
        wait,
        env_expand_path,
        env_expand_args,
    };

    Ok(ret_value)
}

static ENV_VAR: Lazy<Regex> = Lazy::new(|| {
    Regex::new("%([[:word:]]*)%").expect("Invalid Regex")
});

pub fn env_expand(input: &String) -> String {
    // Shamelessly ripped of from:
    // https://users.rust-lang.org/t/expand-win-env-var-in-string/50320/3
    ENV_VAR.replace_all(input.as_str(), |c:&Captures| match &c[1] {
        "" => String::from("%"),
        varname => env::var(varname).unwrap_or("".to_string())
    }).into()
}

pub fn main() {
    // Catch Signals. If signals, set global semaphore.

    let exe_path = env::current_exe().expect("No arg 0? Crazy");
    let shim_path_buf = exe_path.with_extension("shim");

    // println!("Reading exe file at: {:?}", &exe_path);
    // println!("Reading shim file at: {:?}", &shim_path);
    // dbg!(env::vars());

    let shim_path = shim_path_buf.as_path();

    let config = read_config(shim_path).expect(format!("Error reading file: {}", shim_path.display()).as_str());

    // dbg!(&config);

    let path: String = if config.env_expand_path {
        env_expand(&config.path)
    } else {
        config.path
    };

    let args = if config.env_expand_args {
        config.args.iter().map(env_expand).collect()
    } else {
        config.args
    };

    let mut cmd = Command::new(path.to_string());
    cmd.args(args);

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

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    if config.hidden {
        cmd.creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
    }

    let mut child = cmd.spawn().expect(format!("SS: Failed to execute command {}", path).as_str());

    if !config.wait {
        process::exit(0);
    }

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
