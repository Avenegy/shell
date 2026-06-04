use std::io;
use std::process::Command;
use std::io::Write;
use users::{get_user_by_uid, get_current_uid};
use colored::Colorize;

fn main() {
    let user = get_user_by_uid(get_current_uid()).unwrap();
    loop {
        let path = std::env::current_dir().unwrap();
        let part_path: Vec<&str> = path.to_str().unwrap().split('/').collect(); 
        print!("[{} #{}] ", 
        part_path[part_path.len().saturating_sub(2)..].join("/").green(), 
        user.name().to_string_lossy().red());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() { continue; }
        match parts[0] {
            "cd" => {
                let path = parts.get(1).map(|s| *s).unwrap_or("~");
                let path = if path == "~" {
                    std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
                } else {
                    path.to_string()
                };
                std::env::set_current_dir(&path).unwrap_or_else(|_| {
                    println!("cd: no such directory!");
                });
            },
            "pwd" => {
                println!("{}", std::env::current_dir().unwrap().display());
            },

            "exit" => break,

            cmd => {
                match Command::new(cmd).args(&parts[1..]).spawn() {
                    Ok(mut c) => { c.wait().ok(); },
                    Err(_) => println!("command not found: {}", cmd),
                }
            }
        }

    }
}
