use std::io::{self, BufRead, BufReader, IsTerminal};
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
            short: char,
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
                            },
                            ArgSpec {
                                name: "help",
                                short: 'h',
                                long: "--help",
                                kind: ArgKind::Flag
                            }
                        ),*
                    ]
                ),*
            }
        }
    };
}

// TODO: Implement from_str and to_str directly in macro
define_args! {
    Add: {
        tag: {
            short: 't',
            long: "tag",
            kind: Option,
        },
    },
    Log: {
        tag: {
             short: 't',
             long: "tag",
             kind: Option,
         },
         done: {
             short: 'd',
             long: "done",
             kind: Flag,
         },
         undone: {
             short: 'u',
             long: "undone",
             kind: Flag,
         }
    },
    Remove: {
        done: {
            short: 'd',
            long: "done",
            kind: Flag,
        },
    },
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
            "--help" | "help" | "-h" => Ok(Subcommand::Help),
            "--version" | "v" => Ok(Subcommand::Version),
            _ => Err(format!("unknown subcommand \"{s}\"")),
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
    pub fn get_option(args: &[Arg], option_name: &String) -> Option<String> {
        args.iter().find_map(|entry| match entry {
            Arg::Option { name, value } if name == option_name => Some(value.clone()),
            _ => None,
        })
    }
    pub fn get_flag(args: &[Arg], flag_name: &String) -> bool {
        args.iter()
            .any(|entry| matches!(entry, Arg::Flag(str) if str == flag_name))
    }
}

#[derive(Debug, PartialEq)]
pub struct CommandLine {
    pub subcommand: Subcommand,
    pub args: Vec<Arg>,
}

#[derive(Debug)]
enum ParseError {
    UnknownArg(String),
    Missing(String),
}

impl CommandLine {
    pub fn new(tokens: Vec<String>) -> Result<CommandLine, Box<dyn std::error::Error>> {
        if tokens.len() == 1 {
            return Err("Invalid use".into()); // TODO: create an error enum and
        }

        let subcommand = match tokens.get(1) {
            Some(token) => match Subcommand::from_str(token) {
                Ok(cmd) => cmd,
                Err(_) => return Err(format!("unknown subcommand \"{token}\"").into()),
            },
            None => return Err("command not found".into()),
        };

        // args without program name and subcommand
        let mut rem_args = tokens[2..].to_vec();

        if let Some(stdin_args) = read_stdin() {
            println!("{:?}", stdin_args);
            rem_args.extend(stdin_args);
        }

        let arg_spec = get_arg_specs_for(subcommand);

        let args = match Self::parse_args(rem_args, arg_spec) {
            Ok(args) => args,
            Err(ParseError::Missing(arg)) => {
                return Err(format!("switch \"{arg}\" requires a value").into());
            }
            Err(ParseError::UnknownArg(arg)) => {
                return Err(format!("unknown argument \"{arg}\" for {subcommand:#?}").into());
            }
        };

        Ok(CommandLine { subcommand, args })
    }

    // TODO: Refactor this piece of shit
    fn parse_args(
        tokens: Vec<String>,
        arg_spec: &'static [ArgSpec],
    ) -> Result<Vec<Arg>, ParseError> {
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
            let token = &tokens[i];
            if let Some(arg_name) = token.strip_prefix("--") {
                match flags.iter().find(|flag| flag.long == arg_name) {
                    Some(str) => {
                        args.push(Arg::Flag(str.name.to_string()));
                    }
                    None => match options.iter().find(|opt| opt.long == arg_name) {
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
                                None => return Err(ParseError::Missing(arg_name.to_string())),
                            }
                        }
                        None => return Err(ParseError::UnknownArg(arg_name.to_string())),
                    },
                }
            } else if let Some(arg) = token.strip_prefix("-") {
                for a in arg.chars() {
                    match flags.iter().find(|flag| flag.short == a) {
                        Some(str) => {
                            args.push(Arg::Flag(str.name.to_string()));
                        }
                        None => match options.iter().find(|opt| opt.short == a) {
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
                                    None => return Err(ParseError::Missing(a.to_string())),
                                }
                            }
                            None => return Err(ParseError::UnknownArg(a.to_string())),
                        },
                    }
                }
            } else {
                args.push(Arg::Value(token.to_string()));
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

    #[test]
    fn get_args_concatenated() {
        let input = vec!["marc", "log", "-ud"]
            .iter()
            .map(|e| e.to_string())
            .collect();

        let cmd_line = CommandLine::new(input);

        let crt_cmd_line = CommandLine {
            subcommand: Subcommand::Log,
            args: vec![
                Arg::Flag("undone".to_string()),
                Arg::Flag("done".to_string()),
            ],
        };

        assert_eq!(cmd_line.unwrap(), crt_cmd_line);
    }

    #[test]
    fn err_on_unknow_args() {
        let input = vec!["marc", "log", "--pippo"]
            .iter()
            .map(|e| e.to_string())
            .collect();

        let cmd_line = CommandLine::new(input);
        assert!(cmd_line.is_err());
    }

    #[test]
    fn err_on_missing_values() {
        let input = vec!["marc", "add", "--tag"]
            .iter()
            .map(|e| e.to_string())
            .collect();

        let cmd_line = CommandLine::new(input);
        assert!(cmd_line.is_err());
    }
}

pub fn read_stdin() -> Option<Vec<String>> {
    if !io::stdin().is_terminal() {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin.lock());

        return Some(
            reader
                .lines() // TODO: Currently parsing with lines. To implement a parser similar to the Unix one
                .map(|line| line.unwrap_or_default().trim().to_string())
                .filter(|line| !line.is_empty()) // Skip empty lines
                .collect(),
        );
    }

    None
}
