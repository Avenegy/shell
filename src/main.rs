use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, InputMode, Prompt, Result};
use std::env::{current_dir, set_current_dir};
use std::io::Write;
use std::io::{self};
use std::process::Command;
use users::{get_current_uid, get_user_by_uid};

fn main() -> Result<()> {
    let user =
        get_user_by_uid(get_current_uid()).expect("programm panicked at: can't get the user UUID");
    let mut path = current_dir().expect("programm panicked at: can't get current directory");
    // let mut history: Vec<String> = vec![];

    let mut rl = DefaultEditor::new()?;

    loop {
        let path_str = path.to_string_lossy();
        let part_path: Vec<&str> = path_str.split('/').collect();

        let prompt = format!(
            "[{} #{}] ",
            part_path[part_path.len().saturating_sub(2)..]
                .join("/")
                .green(),
            user.name().to_string_lossy().red()
        );
        let input = rl.readline(&prompt);

        match input {
            Ok(line) => {
                let line = line.trim();
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.is_empty() {
                    continue;
                }

                match parts[0] {
                    "cd" => {
                        let cd_path = parts.get(1).copied().unwrap_or("~");
                        let cd_path = if cd_path == "~" {
                            std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
                        } else {
                            cd_path.to_string()
                        };
                        match set_current_dir(&cd_path) {
                            Ok(_) => {
                                path = current_dir().unwrap_or_else(|_| {
                            println!(
                                "pwd: failed to read current directory, staying at last known path"
                            );
                            return path;
                        });
                            }
                            Err(_) => {
                                println!("cd: no such file in directory")
                            }
                        }
                    }
                    "pwd" => {
                        println!("{}", path.display());
                    }

                    "exit" => break Ok(()),

                    cmd => match Command::new(cmd).args(&parts[1..]).spawn() {
                        Ok(mut c) => {
                            c.wait().ok();
                        }
                        Err(_) => println!("command not found: {}", cmd),
                    },
                }
            }
            Err(ReadlineError::Interrupted) => {
                continue;
            }
            Err(ReadlineError::Eof) => {
                break Ok(());
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break Ok(());
            }
        }
    }
}
