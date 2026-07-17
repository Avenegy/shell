use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use std::env::{current_dir, set_current_dir};
use std::process::{ChildStdout, Command, Stdio};
use users::{get_current_uid, get_user_by_uid};

fn main() -> Result<()> {
    let user =
        get_user_by_uid(get_current_uid()).expect("programm panicked at: can't get the user UID");
    let mut path = current_dir().expect("programm panicked at: can't get current directory");
    let mut rl = DefaultEditor::new()?;
    rl.load_history("history.txt").ok();

    'shell: loop {
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
                let parts: Vec<&str> = line.split("|").collect();
                let mut child_parts = vec![];
                for item in parts.iter() {
                    let item: Vec<_> = item.trim().split_whitespace().collect();
                    child_parts.push(item);
                }

                child_parts.retain(|item| !item.is_empty());

                if child_parts.is_empty() {
                    continue;
                } else {
                    rl.add_history_entry(line)?;
                }
                if child_parts.len() == 1 {
                    match child_parts[0][0] {
                        "cd" => {
                            let cd_path = child_parts[0].get(1).copied().unwrap_or("~");
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

                        cmd => match Command::new(cmd).args(&child_parts[0][1..]).spawn() {
                            Ok(mut c) => {
                                c.wait().ok();
                            }
                            Err(_) => println!("command not found: {}", cmd),
                        },
                    }
                } else {
                    let mut pipe_out: Option<ChildStdout> = None;
                    let mut active_childs = vec![];
                    for (i, item) in child_parts.iter().enumerate() {
                        if i == 0 {
                            match Command::new(item[0])
                                .args(&item[1..])
                                .stdout(Stdio::piped())
                                .spawn()
                            {
                                Ok(mut child) => {
                                    pipe_out = Some(
                                        child.stdout.take().expect("Failed to open pipe stdout"),
                                    );
                                    active_childs.push(child);
                                }
                                Err(_) => {
                                    println!("command not found: {}", item[0]);
                                    continue 'shell;
                                }
                            }
                        } else if i == child_parts.len() - 1 {
                            match Command::new(item[0])
                                .args(&item[1..])
                                .stdin(Stdio::from(pipe_out.take().unwrap()))
                                .spawn()
                            {
                                Ok(child) => {
                                    active_childs.push(child);
                                }
                                Err(_) => {
                                    println!("command not found: {}", item[0]);
                                    continue 'shell;
                                }
                            }
                        } else {
                            match Command::new(item[0])
                                .args(&item[1..])
                                .stdin(Stdio::from(pipe_out.take().unwrap()))
                                .stdout(Stdio::piped())
                                .spawn()
                            {
                                Ok(mut child) => {
                                    pipe_out = Some(
                                        child.stdout.take().expect("Failed to open pipe stdout"),
                                    );
                                    active_childs.push(child);
                                }
                                Err(_) => {
                                    println!("command not found: {}", item[0]);
                                    continue 'shell;
                                }
                            }
                        }
                    }
                    for mut c in active_childs {
                        c.wait().ok();
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                continue;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    let _ = rl.save_history("history.txt");
    Ok(())
}
