use std::env;
use std::error::Error;
use std::fs::{self};
use std::io::Write;
use std::path::{Path, PathBuf};

mod temporal;

enum Cmd {
    List,
    Add { note: String },
    Help,
    Version,
}

impl Cmd {
    fn new(args: &[String]) -> Result<Cmd, String> {
        if args.len() == 1 {
            return Ok(Cmd::Help);
        }

        match args[1].as_str() {
            "--version" | "-v" => Ok(Cmd::Version),
            "--help" | "-h" => Ok(Cmd::Help),
            "ls" | "list" => Ok(Cmd::List),
            "add" => {
                let content = args.get(2);

                match content {
                    Some(str) => Ok(Cmd::Add { note: str.clone() }),
                    None => Err(format!("Add command needs content")),
                }
            }
            _ => Err(format!("Unknown command: {}", args[1])),
        }
    }

    fn execute(&self) -> Result<(), Box<dyn Error>> {
        match self {
            Cmd::List => list(),
            Cmd::Add { note } => add(note),
            Cmd::Version => {
                let env = env!("CARGO_PKG_VERSION");
                let name = env!("CARGO_PKG_NAME");
                println!("{} version {}", name, env);
                Ok(())
            }
            Cmd::Help => help(),
        }
    }
}

pub fn run(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let cmd = Cmd::new(&args)?;
    cmd.execute()?;

    Ok(())
}

/// Help command -- Displays all the commands, their usage and a short description
fn help() -> Result<(), Box<dyn Error>> {
    println!("Marc usage");
    Ok(())
}

/// Add command -- Adds an entry to a list
fn add(content: &String) -> Result<(), Box<dyn Error>> {
    let file_path = get_file_path()?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;
    writeln!(file, "- [ ] {content}")?;

    Ok(())
}

// List command -- Shows notes for a given list
fn list() -> Result<(), Box<dyn Error>> {
    let file_path = get_file_path()?;
    let content = fs::read_to_string(file_path)?;
    println!("{content}");
    Ok(())
}

fn get_file_path() -> Result<String, Box<dyn Error>> {
	let today = temporal::today();

    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE")) // Windows fallback TODO Testing
        .expect("Home directory not set");

    let dir_path = PathBuf::from(&home)
        .join("marc")
        .join(today.month.to_string());

    let path = Path::new(&dir_path);
    if !path.exists() {
        fs::create_dir_all(&dir_path)?;
    }

    let file_path = PathBuf::from(&dir_path)
        .join(format!("{}.md", today.day));

    Ok(file_path.to_string_lossy().to_string())
}
