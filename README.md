<h1 align="center">Rust Shell</h1>
<p align="center"> A basic shell written in rust as a learning project.</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-learning%20project-orange?style=flat-square&logo=rust" />
  <img src="https://img.shields.io/badge/status-complete-green?style=flat-square" />
</p>

---

## About

A Unix shell built from scratch in Rust to learn systems programming concepts like process management, I/O redirection, and pipelines. It implements a REPL loop with a colored prompt, persistent command history via `rustyline`, and support for chaining commands together with pipes and redirects the core mechanics behind how a real shell talks to the OS.

This is not meant to replace `bash` or `zsh`. It's a learning project focused on understanding how shells actually work under the hood.

## Features

- **Interactive REPL** with a colored prompt showing the current directory and username
- **Command history** — persisted across sessions (`history.txt`), navigable with arrow keys via `rustyline`
- **Built-in commands** — `cd`, `pwd`, `exit`
- **Pipes (`|`)** — chain any number of commands together (`cmd1 | cmd2 | cmd3`)
- **I/O redirection**:
  - `>` — redirect stdout to a file (overwrite)
  - `>>` — redirect stdout to a file (append)
  - `<` — redirect stdin from a file
- **Basic error handling** — reports missing commands, unreadable/unwritable files, and invalid `cd` paths without crashing the shell

## Installation


```bash
git clone https://github.com/Avenegy/rust-shell.git
cd rust-shell
cargo build --release
```

The binary will be available at `target/release/<binary-name>`.

### Dependencies

- [`rustyline`](https://crates.io/crates/rustyline) — line editing, history, arrow-key navigation
- [`colored`](https://crates.io/crates/colored) — colored terminal output
- [`users`](https://crates.io/crates/users) — fetching the current system user (Unix-only)

> **Note:** this shell relies on Unix-specific APIs (via the `users` crate) and is not expected to build or run on Windows.

## Usage

Run the compiled binary and start typing commands like you would in any shell:

```bash
$ ./rust-shell
[project @dmytro] ls -la
[project @dmytro] cd ~/code
[project @dmytro] pwd
```

### Pipes

```bash
[project @dmytro] cat file.txt | grep rust | wc -l
```

### Redirects

```bash
# overwrite a file with command output
[project @dmytro] echo "hello world" > output.txt

# append to a file
[project @dmytro] echo "another line" >> output.txt

# read stdin from a file
[project @dmytro] sort < unsorted.txt

# combine pipes and redirects
[project @dmytro] cat access.log | grep ERROR > errors.txt
```

## Built-in commands

| Command | Description |
|---|---|
| `cd [path]` | Changes the current directory. Defaults to `$HOME` if no path is given. |
| `pwd` | Prints the current working directory. |
| `exit` | Exits the shell and saves command history. |

Everything else is spawned as an external process via `std::process::Command`.

## Known limitations

Since this is a learning project, several things a "real" shell handles aren't implemented yet:

- **No quoting support** — `echo "hello world"` is split on whitespace into separate arguments rather than treated as one string
- **No environment variable expansion** — `$HOME`, `$PATH`, etc. aren't substituted in commands
- **No background execution** — there's no `&` support to run processes asynchronously
- **No globbing** — wildcards like `*.txt` aren't expanded
- **No command substitution** — no `$(...)` or backticks
- **Limited built-ins** — no `export`, `alias`, `history`, or job control (`jobs`, `fg`, `bg`)

<br>

*This project is complete, no further development planned.*
