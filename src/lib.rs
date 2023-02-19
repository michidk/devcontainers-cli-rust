use std::{path::Path, process::Command};

use thiserror::Error;


#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("UTF8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
}

type Result<T> = std::result::Result<T, Error>;


/// read the devcontainer configuration for the given path (or current directory, if none is given)
/// returns the unprocessed JSON string
pub fn read_configuration(path: Option<&Path>) -> Result<String>
{
    let mut args = vec!["read-configuration".to_string()];

    if let Some(path) = path {
        if !path.exists() {
            return Err(Error::InvalidPath(path.to_string_lossy().to_string()));
        }
        args.push(path.to_string_lossy().into_owned());
    }

    let executable = if cfg!(target_os = "windows") {
        "devcontainer.cmd"
    } else {
        "devcontainer"
    };

    let stdout = Command::new(executable)
        .args(args)
        .output()?.stdout;
    let result = std::str::from_utf8(&stdout)?.trim().to_string();

    remove_node_stuff(result)
}

fn remove_node_stuff(input: String) -> Result<String> {
    let split: Vec<&str> = input.split("\n").collect();
    let mut result = String::new();
    for line in split {
        let line = line.replace("\r", "");
        if line.starts_with("(node:") {
            continue;
        }
        if line.starts_with("(Use `Code --trace-deprecation ...") {
            continue;
        }
        if line.starts_with("[") && (line.ends_with("x64.") || line.ends_with("x86.")) {
            continue;
        }
        result.push_str(&line);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::{read_configuration};

    // These tests may not execute on all machines
    #[test]
    fn test_read_configuration() {
        let ws = std::env::current_dir().unwrap();

        // println!("Current config: {}", read_configuration(None).unwrap());

        let expected = r#"{"configuration":{"name":"Rust","image":"mcr.microsoft.com/devcontainers/rust:0-1-bullseye","configFilePath":{"$mid":1,"path":"/C:/Users/michi/development/private/devcontainers-cli-rust/.devcontainer/devcontainer.json","scheme":"vscode-fileHost"}},"workspace":{"workspaceFolder":"/workspaces/devcontainers-cli-rust","workspaceMount":"type=bind,source=C:\\Users\\michi\\development\\private\\devcontainers-cli-rust,target=/workspaces/devcontainers-cli-rust,consistency=consistent"}}"#;
        assert_eq!(read_configuration(None).unwrap(), expected);
        // assert_eq!(read_configuration(Some(ws.as_path())).unwrap(), expected);
    }

}
