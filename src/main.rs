use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use std::env::{current_dir, set_current_dir};
use std::fs::{File, OpenOptions};
use std::process::{ChildStdout, Command, Stdio};
use users::{get_current_uid, get_user_by_uid};

struct ParsedCommand<'a> {
    args: Vec<&'a str>,
    stdin_redirect: Option<&'a str>,
    stdout_redirect: Option<(&'a str, bool)>,
}

fn parse_redirects<'a>(item: &[&'a str]) -> ParsedCommand<'a> {
    let mut args = Vec::new();
    let mut stdin_redirect = None;
    let mut stdout_redirect = None;

    let mut i = 0;
    while i < item.len() {
        match item[i] {
            "<" => {
                if let Some(filename) = item.get(i + 1) {
                    stdin_redirect = Some(*filename);
                    i += 2;
                } else {
                    println!("syntax error: expected filename after '<'");
                    i += 1;
                }
            }
            ">" => {
                if let Some(filename) = item.get(i + 1) {
                    stdout_redirect = Some((*filename, false));
                    i += 2;
                } else {
                    println!("syntax error: expected filename after '>'");
                    i += 1;
                }
            }
            ">>" => {
                if let Some(filename) = item.get(i + 1) {
                    stdout_redirect = Some((*filename, true));
                    i += 2;
                } else {
                    println!("syntax error: expected filename after '>>'");
                    i += 1;
                }
            }
            other => {
                args.push(other);
                i += 1;
            }
        }
    }

    ParsedCommand {
        args,
        stdin_redirect,
        stdout_redirect,
    }
}

fn open_stdout_file(filename: &str, append: bool) -> std::io::Result<File> {
    if append {
        OpenOptions::new().create(true).append(true).open(filename)
    } else {
        File::create(filename)
    }
}

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
                    let parsed = parse_redirects(&child_parts[0]);

                    if parsed.args.is_empty() {
                        continue;
                    }

                    match parsed.args[0] {
                        "cd" => {
                            let cd_path = parsed.args.get(1).copied().unwrap_or("~");
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

                        cmd => {
                            let mut command = Command::new(cmd);
                            command.args(&parsed.args[1..]);

                            if let Some(in_file) = parsed.stdin_redirect {
                                match File::open(in_file) {
                                    Ok(file) => {
                                        command.stdin(Stdio::from(file));
                                    }
                                    Err(_) => {
                                        println!("cannot open '{}' for reading", in_file);
                                        continue;
                                    }
                                }
                            }

                            if let Some((out_file, append)) = parsed.stdout_redirect {
                                match open_stdout_file(out_file, append) {
                                    Ok(file) => {
                                        command.stdout(Stdio::from(file));
                                    }
                                    Err(_) => {
                                        println!("cannot open '{}' for writing", out_file);
                                        continue;
                                    }
                                }
                            }

                            match command.spawn() {
                                Ok(mut c) => {
                                    c.wait().ok();
                                }
                                Err(_) => println!("command not found: {}", cmd),
                            }
                        }
                    }
                } else {
                    let mut pipe_out: Option<ChildStdout> = None;
                    let mut active_childs = vec![];
                    let n = child_parts.len();

                    for (i, item) in child_parts.iter().enumerate() {
                        let parsed = parse_redirects(item);

                        if parsed.args.is_empty() {
                            continue 'shell;
                        }

                        let mut command = Command::new(parsed.args[0]);
                        command.args(&parsed.args[1..]);

                        if let Some(in_file) = parsed.stdin_redirect {
                            match File::open(in_file) {
                                Ok(file) => {
                                    command.stdin(Stdio::from(file));
                                }
                                Err(_) => {
                                    println!("cannot open '{}' for reading", in_file);
                                    continue 'shell;
                                }
                            }
                        } else if i != 0 {
                            command.stdin(Stdio::from(pipe_out.take().unwrap()));
                        }

                        if let Some((out_file, append)) = parsed.stdout_redirect {
                            match open_stdout_file(out_file, append) {
                                Ok(file) => {
                                    command.stdout(Stdio::from(file));
                                }
                                Err(_) => {
                                    println!("cannot open '{}' for writing", out_file);
                                    continue 'shell;
                                }
                            }
                        } else if i != n - 1 {
                            command.stdout(Stdio::piped());
                        }

                        match command.spawn() {
                            Ok(mut child) => {
                                if parsed.stdout_redirect.is_none() && i != n - 1 {
                                    pipe_out = Some(
                                        child.stdout.take().expect("Failed to open pipe stdout"),
                                    );
                                }
                                active_childs.push(child);
                            }
                            Err(_) => {
                                println!("command not found: {}", parsed.args[0]);
                                continue 'shell;
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
