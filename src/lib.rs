use std::error::Error;

use rusqlite::Connection;

enum Cmd {
    List {
        list_name: Option<String>,
    },
    Add {
        content: String,
        list_name: Option<String>,
    },
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
            "ls" | "list" => Ok(Cmd::List {
                list_name: args.get(2).cloned(),
            }),
            "add" => {
                let content = args.get(2);
                let list_name = args.get(3).cloned();

                match content {
                    Some(str) => Ok(Cmd::Add {
                        content: str.clone(),
                        list_name,
                    }),
                    None => Err(format!("Add command needs content")),
                }
            }
            _ => Err(format!("Unknown command: {}", args[1])),
        }
    }

    fn execute(&self) -> Result<(), Box<dyn Error>> {
        match self {
            Cmd::List { list_name } => list(list_name),
            Cmd::Add { content, list_name } => add(content, list_name),
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

struct DB {}
impl DB {
    pub fn connect() -> Result<Connection, Box<dyn Error>> {
        let db_path = format!("{}/marc/", env!("HOME"));
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS list (
				id INTEGER PRIMARY KEY NOT NULL,
				name TEXT NOT NULL UNIQUE
			)",
            (),
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS note (
				id INTEGER PRIMARY KEY NOT NULL,
				content TEXT NOT NULL,
				list_id INTEGER NOT NULL,
				FOREIGN KEY (list_id) REFERENCES list(id)
			)",
            (),
        )?;

        conn.execute("INSERT OR IGNORE INTO list (name) VALUES ('default')", ())?;

        Ok(conn)
    }
}

/// Add command -- Adds an entry to a list
fn add(content: &String, list_name: &Option<String>) -> Result<(), Box<dyn Error>> {
    let conn = DB::connect()?;
    let list_name = list_name.as_deref().unwrap_or("default");

    let added = conn.execute(
	    "INSERT INTO note (content, list_id)
	    VALUES (?1, (SELECT id FROM list WHERE name = ?2))",
        (content, list_name),
    )?;

    match added {
        0 => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to insert note",
        ))),
        _ => Ok(()),
    }
}

// List command -- Shows notes for a given list
fn list(list_name: &Option<String>) -> Result<(), Box<dyn Error>> {
    let conn = DB::connect()?;
    let list_name = list_name.as_deref().unwrap_or("default");

    let mut stmt = conn.prepare(
        "SELECT content FROM note WHERE list_id = (SELECT id FROM list WHERE name = ?1)",
    )?;

    let note_iter = stmt.query_map([list_name], |row| Ok(row.get::<_, String>(0)?))?;

    for note in note_iter {
        println!("{}", note?);
    }

    Ok(())
}
