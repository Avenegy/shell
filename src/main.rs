use std::io;
use std::process::Command;
use std::io::Write;

fn main() {
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() { continue; }
        let rest = &parts[1..];
        let name = rest.join(" ");
        match parts[0] {
            "cd" => {
                let path = parts.get(1).unwrap_or(&"~");
                std::env::set_current_dir(path).unwrap_or_else(|_| {
                    println!("cd: no such directory!");
                })
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

        //println!("got {input}")
    }
}
