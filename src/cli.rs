use std::str::FromStr;

macro_rules! define_args {
    {
        $(
            $cmd:ident: {
                $(
                    $arg_name:ident: {
                        short: $short:literal,
                        long: $long:literal,
                        kind: $kind:ident,
                    }
                ),* $(,)?
            }
        ),* $(,)?
    } => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Subcommand {
            $($cmd),*
        }

        #[derive(Debug, Clone, Copy, PartialEq)]
        enum ArgKind {
            Flag,
            Option,
        }

        #[derive(Debug, Clone)]
        struct ArgSpec {
            name: &'static str,
            short: &'static str,
            long: &'static str,
            kind: ArgKind,
        }

        fn get_arg_specs_for(cmd: Subcommand) -> &'static [ArgSpec] {
            match cmd {
                $(
                    Subcommand::$cmd => &[
                        $(
                            ArgSpec {
                                name: stringify!($arg_name),
                                short: $short,
                                long: $long,
                                kind: ArgKind::$kind,
                            }
                        ),*
                    ]
                ),*
            }
        }
    };
}

// TODO: Implement from_str directly in macro
define_args! {
    Add: {
        tag: {
            short: "t",
            long: "tag",
            kind: Option,
        },
    },
    Log: {
        tag: {
             short: "t",
             long: "tag",
             kind: Option,
         },
         completed: {
             short: "c",
             long: "completed",
             kind: Flag,
         }
    },
    Remove: {},
    Edit: {},
    Help: {},
    Done: {},
    Version: {}
}

impl FromStr for Subcommand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "add" => Ok(Subcommand::Add),
            "rm" => Ok(Subcommand::Remove),
            "log" => Ok(Subcommand::Log),
            "edit" => Ok(Subcommand::Edit),
            "done" => Ok(Subcommand::Done),
            "--help" => Ok(Subcommand::Help),
            "--version" => Ok(Subcommand::Version),
            _ => return Err(format!("Unknown subcommand: {}", s)),
        }
    }
}

/// An argument is a flag or an option for the subcommand
#[derive(Debug, PartialEq)]
pub enum Arg {
    Option { name: String, value: String }, // The option can accept a value
    Flag(String),                           // Just a switch
    Value(String),                          // Argument containing a value
}

impl Arg {
    pub fn get_option(args: &Vec<Arg>, option_name: &String) -> Option<String> {
        args.iter().find_map(|entry| match entry {
            Arg::Option { name, value } if name == option_name => Some(value.clone()),
            _ => None,
        })
    }
    pub fn get_flag(args: &Vec<Arg>, flag_name: &String) -> bool {
        args.iter()
            .any(|entry| matches!(entry, Arg::Flag(str) if str == flag_name))
    }
}

#[derive(Debug, PartialEq)]
pub struct CommandLine {
    pub subcommand: Subcommand,
    pub args: Vec<Arg>,
}

impl CommandLine {
    pub fn new(tokens: Vec<String>) -> Result<CommandLine, Box<dyn std::error::Error>> {
        if tokens.len() == 1 {
            return Err("Invalid use".into());
        }

        let subcommand = match tokens.get(1) {
            Some(token) => match Subcommand::from_str(token) {
                Ok(cmd) => cmd,
                Err(_) => return Err("invalid subcommand".into()),
            },
            None => return Err("command not found".into()),
        };

        // args without program name and subcommand
        let rem_args = tokens[2..].to_vec();
        let arg_spec = get_arg_specs_for(subcommand);

        let args = Self::parse_args(rem_args, arg_spec).expect("Ahioo");

        Ok(CommandLine { subcommand, args })
    }

    // TODO: Refactor this piece of shit
    fn parse_args(
        tokens: Vec<String>,
        arg_spec: &'static [ArgSpec],
    ) -> Result<Vec<Arg>, Box<dyn std::error::Error>> {
        let flags: Vec<&ArgSpec> = arg_spec
            .iter()
            .filter(|f| f.kind == ArgKind::Flag)
            .collect();
        let options: Vec<&ArgSpec> = arg_spec
            .iter()
            .filter(|o| o.kind == ArgKind::Option)
            .collect();

        let mut args: Vec<Arg> = vec![];

        let mut i = 0;
        while i < tokens.len() {
            let t = &tokens[i];
            // long format
            if t.starts_with("--") {
                let entry = &t[2..];

                // flags
                match flags.iter().find(|flag| flag.long == entry) {
                    Some(str) => {
                        args.push(Arg::Flag(str.name.to_string()));
                        i += 1;
                        continue;
                    }

                    None => (),
                }

                // options
                match options.iter().find(|opt| opt.long == entry) {
                    Some(str) => {
                        let next_token = tokens.get(i + 1);

                        match next_token {
                            Some(next) => {
                                i += 1;
                                args.push(Arg::Option {
                                    name: str.name.to_string(),
                                    value: next.to_string(),
                                });
                            }
                            None => (), // Handle error "option should have a value"
                        }
                    }
                    None => (),
                }
            } else if t.starts_with("-") {
                // TODO: Implement concatened feature
                // short format
                let entry = &t[1..];

                // flags
                match flags.iter().find(|flag| flag.short == entry) {
                    Some(str) => {
                        args.push(Arg::Flag(str.name.to_string()));
                        i += 1;
                        continue;
                    }
                    None => (),
                }

                match options.iter().find(|opt| opt.short == entry) {
                    Some(str) => {
                        let next_token = tokens.get(i + 1);

                        match next_token {
                            Some(next) => {
                                i += 1;
                                args.push(Arg::Option {
                                    name: str.name.to_string(),
                                    value: next.to_string(),
                                });
                            }
                            None => (), // Handle error "option should have a value"
                        }
                    }
                    None => (),
                }
            } else {
                args.push(Arg::Value(t.to_string()));
            }
            i += 1;
        }
        Ok(args)
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::{Arg, CommandLine, Subcommand};

    #[test]
    fn get_args_long() {
        let input = vec!["marc", "add", "--tag", "test", "should work"]
            .iter()
            .map(|e| e.to_string())
            .collect();

        let cmd_line = CommandLine::new(input);

        let crt_cmd_line = CommandLine {
            subcommand: Subcommand::Add,
            args: vec![
                Arg::Option {
                    name: "tag".to_string(),
                    value: "test".to_string(),
                },
                Arg::Value("should work".to_string()),
            ],
        };

        assert_eq!(cmd_line.unwrap(), crt_cmd_line);
    }

    #[test]
    fn get_args_short() {
        let input = vec!["marc", "add", "-t", "test", "should work"]
            .iter()
            .map(|e| e.to_string())
            .collect();

        let cmd_line = CommandLine::new(input);

        let crt_cmd_line = CommandLine {
            subcommand: Subcommand::Add,
            args: vec![
                Arg::Option {
                    name: "tag".to_string(),
                    value: "test".to_string(),
                },
                Arg::Value("should work".to_string()),
            ],
        };

        assert_eq!(cmd_line.unwrap(), crt_cmd_line);
    }
}
