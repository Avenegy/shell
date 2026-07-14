use colored::Colorize;
use std::env::{current_dir, set_current_dir};
use std::io::Write;
use std::io::{self};
use std::process::Command;
use users::{get_current_uid, get_user_by_uid};

fn main() {
    let user =
        get_user_by_uid(get_current_uid()).expect("programm panicked at: can't get the user UUID");
    let mut path = current_dir().expect("programm panicked at: can't get current directory");
    let mut history: Vec<String> = vec![];
    loop {
        let path_str = path.to_string_lossy();
        let part_path: Vec<&str> = path_str.split('/').collect();
        print!(
            "[{} #{}] ",
            part_path[part_path.len().saturating_sub(2)..]
                .join("/")
                .green(),
            user.name().to_string_lossy().red()
        );
        match io::stdout().flush() {
            Ok(_) => {}
            Err(e) => {
                eprint!("Error: {}", e);
                break;
            }
        };

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        };
        let input = input.trim();
        let history_input = String::from(input);
        history.push(history_input);

        let parts: Vec<&str> = input.split_whitespace().collect();
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

            "exit" => break,

            cmd => match Command::new(cmd).args(&parts[1..]).spawn() {
                Ok(mut c) => {
                    c.wait().ok();
                }
                Err(_) => println!("command not found: {}", cmd),
            },
        }
    }
}
