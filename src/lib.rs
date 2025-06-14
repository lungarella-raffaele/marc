enum Cmd {
    List { list_name: String },
    Add,
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
            "ls" | "list" => {
                match args.get(2) {
                    Some(str) => Ok(Cmd::List {
                        list_name: str.clone(),
                    }),
                    None => Err(format!("List needs a list name")),
                }
            },
            "-m" => Ok(Cmd::Add),
            _ => Err(format!("Unknown command: {}", args[1])),
        }
    }

    fn execute(&self) {
        match self {
            Cmd::List { list_name } => println!("{}", list_name),
            Cmd::Add  => todo!(),
            Cmd::Version => {
                let env = env!("CARGO_PKG_VERSION");
                let name = env!("CARGO_PKG_NAME");
                println!("{} version {}", name, env);
            }
            Cmd::Help => help(),
        }
    }
}

pub fn run(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let cmd = Cmd::new(&args)?;
    cmd.execute();

    Ok(())
}

fn help() {
    println!("Marc usage")
}
