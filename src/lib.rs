use std::collections::hash_map::DefaultHasher;
use std::env;
use std::error::Error;
use std::fs::{self};
use std::hash::{Hash, Hasher};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

pub fn run(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() == 1 {
        help()?;
        return Ok(());
    }

    match args[1].as_str() {
        "add" => {
            if args.len() < 3 {
                // TODO Instead of showing error prompt user to add TODOS
                return Err("'add' command requires at least one entry".into());
            }

            add(&args[2..])?;
        }
        "log" => {
            log()?;
        }
        "edit" => {
            edit()?;
        }
        "rm" => {
            let hashes: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();
            rm(&hashes)?
        }
        "done" => {
            if args.len() < 3 {
                return Err(
                    "'done' command requires at least one hash to complete an entry".into(),
                );
            }
            let hashes = &args[2..];
            done(hashes)?;
        }
        "--version" | "-v" => {
            let env = env!("CARGO_PKG_VERSION");
            let name = env!("CARGO_PKG_NAME");
            println!("{} version {}", name, env);
        }
        "--help" | "-h" => {
            help()?;
        }
        _ => {
            return Err(format!("'{}' is not a marc command. See 'marc --help'.", args[1]).into());
        }
    }
    Ok(())
}

pub struct Config {}
impl Config {
    pub fn get_path() -> Result<PathBuf, Box<dyn Error>> {
        let home_dir = env::var("HOME")
            .or_else(|_| env::var("USERPROFILE"))
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("home directory not set: {}", e),
                )
            })?;

        let mut app_data_dir = PathBuf::from(&home_dir);
        app_data_dir.push("marc");

        const DB_FILE_NAME: &str = "marc.json";

        let path = app_data_dir.join(DB_FILE_NAME);

        if let Some(parent_dir) = path.parent() {
            if !parent_dir.exists() {
                fs::create_dir_all(parent_dir)?;
            }
        }

        Ok(path)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TodoItem {
    hash: String,
    desc: String,
    is_completed: bool,
    tag: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TodoList {
    items: Vec<TodoItem>,
}

#[derive(Debug)]
enum MarkDoneError {
    NotFound(String),
    AlreadyCompleted(String),
    MultipleMatches(String, Vec<(String, String)>), // prefix, vec of (id, desc)
}

impl TodoList {
    fn new() -> Self {
        TodoList { items: Vec::new() }
    }

    fn load_from_file() -> Result<Self, Box<dyn Error>> {
        let path: PathBuf = Config::get_path()?;

        if !path.exists() {
            return Ok(TodoList::new());
        }

        let data = fs::read_to_string(&path)
            .map_err(|e| format!("error: failed to read todo file: {}", e))?;

        if data.trim().is_empty() {
            return Ok(TodoList::new());
        }

        match serde_json::from_str(&data) {
            Ok(list) => Ok(list),
            Err(e) => Err(format!(
                "error: failed to parse todo file ({}). Error: {}",
                path.display(),
                e
            )
            .into()),
        }
    }

    fn add_item(&mut self, desc: String, tag: Option<String>) {
        let id = Self::generate_short_hash(&desc, &tag);
        let new_item = TodoItem {
            hash: id.clone(),
            desc: desc.clone(),
            is_completed: false,
            tag: Some(tag.unwrap_or("default".to_string())),
        };
        self.items.push(new_item);

        let tag_display = self
            .items
            .last()
            .unwrap()
            .tag
            .as_ref()
            .map(|t| format!(" #{}", t))
            .unwrap_or_default();
        println!("Added: '{}'{} [{}]", desc, tag_display, id);
    }

    fn rm_item(&mut self, hash: &str) -> Option<TodoItem> {
        let matching_items: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.hash.starts_with(hash))
            .map(|(i, _)| i)
            .collect();

        match matching_items.len() {
            0 => None,
            1 => {
                let index = matching_items[0];

                Some(self.items.remove(index))
            }
            _ => None,
        }
    }

    fn save_to_file(&self) -> Result<(), Box<dyn Error>> {
        let path = Config::get_path()?;

        let serialized = serde_json::to_string_pretty(&self)
            .map_err(|e| format!("error: failed to serialize todo data: {}", e))?;
        fs::write(&path, serialized).map_err(|e| {
            format!(
                "error: failed to write to todo file ({}): {}",
                path.display(),
                e
            )
        })?;
        Ok(())
    }

    fn list_items(&self) {
        if self.items.is_empty() {
            println!("No entries");
            return;
        }

        println!("\x1b[1;31m total {}\x1b[0m", self.items.len());

        for item in self.items.iter() {
            let (desc, status) = if item.is_completed {
                (item.desc.clone(), 1)
            } else {
                (item.desc.clone(), 0)
            };

            println!(
                "{} {} {} {}",
                status,
                item.hash,
                item.tag
                    .as_ref()
                    .map_or(String::new(), |tag| format!("\x1b[36m#{}\x1b[0m", tag)),
                desc,
            );
        }
    }

    fn generate_short_hash(desc: &str, tag: &Option<String>) -> String {
        let mut hasher = DefaultHasher::new();
        desc.hash(&mut hasher);
        if let Some(tag_value) = tag {
            tag_value.hash(&mut hasher);
        }
        // Use current timestamp to ensure uniqueness
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .hash(&mut hasher);

        let hash = hasher.finish();
        format!("{:x}", hash)[..7].to_string()
    }

    fn mark_done(&mut self, hash: &str) -> Result<usize, MarkDoneError> {
        let matching_items: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.hash.starts_with(hash))
            .map(|(i, _)| i)
            .collect();

        match matching_items.len() {
            0 => Err(MarkDoneError::NotFound(format!(
                "warning: no todo found with hash' {}'",
                hash
            ))),
            1 => {
                let index = matching_items[0];
                if self.items[index].is_completed {
                    Err(MarkDoneError::AlreadyCompleted(
                        "warning: todo is already completed".to_string(),
                    ))
                } else {
                    self.items[index].is_completed = true;
                    Ok(1)
                }
            }
            _ => {
                let matches: Vec<(String, String)> = matching_items
                    .iter()
                    .map(|&i| (self.items[i].hash.clone(), self.items[i].desc.clone()))
                    .collect();
                Err(MarkDoneError::MultipleMatches(hash.to_string(), matches))
            }
        }
    }
}

/// Help command -- Displays all the commands, their usage and a short description
fn help() -> Result<(), Box<dyn Error>> {
    println!("marc - A simple todo list manager\n");
    println!("USAGE:");
    println!("    marc <COMMAND> [OPTIONS]\n");
    println!("COMMANDS:");
    println!("    add [--tag TAG] <todo>...   Add one or more todos");
    println!("    log                         List all todos");
    println!("    edit                        Interactive edit mode");
    println!("    done <hash>...              Mark todos as complete by hash ID");
    println!("    rm <hash>...                Deletes todos by hash ID");
    println!("    --help, -h                  Show this help message");
    println!("    --version, -v               Show version information\n");
    Ok(())
}

/// The get flag methods finds a certain flag inside the args and returns the value the flag identifies.
/// e.g. if the flag is -t, the command -t my_tag would return my_tag
fn get_flag<'a>(aliases: &'a [&'a str], args: &'a [String]) -> Option<&'a String> {
    args.iter()
        .position(|entry| aliases.contains(&entry.as_str()))
        .and_then(|pos| args.get(pos + 1))
}

/// Add command -- Adds entries to a list
fn add(args: &[String]) -> Result<(), Box<dyn Error>> {
    let mut todo_list = TodoList::load_from_file()?;

    let (tag, todos_start_index) =
        match get_flag(&["-t", "--tag"], args) {
            Some(tag_value) if !tag_value.is_empty() => (Some(tag_value), 2),
            Some(_) => return Err(
                "--tag option requires a non-empty value.\nUsage: marc add --tag <tagname> <todo>"
                    .into(),
            ),
            None => {
                return Err(
                    "--tag option requires a value.\nUsage: marc add --tag <tagname> <todo>".into(),
                );
            }
        };

    if tag.is_some() && args.get(todos_start_index) == None {
        return Err("did not specify the todo content".into());
    }

    // Add todos starting from the determined index
    let todos_to_add = &args[todos_start_index..];
    if todos_to_add.is_empty() {
        return Err(
            "No todos provided after tag option.\nUsage: marc add --tag <tagname> <todo>".into(),
        );
    }

    for todo in todos_to_add {
        if todo.trim().is_empty() {
            return Err("Todo items cannot be empty".into());
        }
        todo_list.add_item(todo.clone(), tag.cloned());
    }

    todo_list.save_to_file()?;
    Ok(())
}

/// List command -- Shows notes for a given list
fn log() -> Result<(), Box<dyn Error>> {
    let todo_list = TodoList::load_from_file()?;
    todo_list.list_items();

    Ok(())
}

fn rm(hash: &[&str]) -> Result<(), Box<dyn Error>> {
    if hash.is_empty() {
        return Err("rm command required an hash".into());
    }
    let mut todo_list = TodoList::load_from_file()?;

    if todo_list.items.is_empty() {
        return Err("No todos to remove".into());
    }

    for prefix in hash {
        if prefix.trim().is_empty() {
            continue;
        };

        todo_list.rm_item(prefix);
    }

    todo_list.save_to_file()?;
    Ok(())
}

/// Interactive edit command -- Opens editor to pick/drop todos
fn edit() -> Result<(), Box<dyn Error>> {
    let mut todo_list = TodoList::load_from_file()?;

    if todo_list.items.is_empty() {
        return Err("No todos to edit! Add some todos first with 'marc add <todo>'".into());
    }

    let mut temp_file = NamedTempFile::new()?;

    for (i, item) in todo_list.items.iter().enumerate() {
        writeln!(temp_file, "pick {} {}", i + 1, item.desc)?;
    }

    writeln!(temp_file, "\n# Interactive todo editing")?;
    writeln!(temp_file, "# Commands:")?;
    writeln!(temp_file, "#   pick, p <todo> = keep the todo")?;
    writeln!(temp_file, "#   drop, d <todo> = remove the todo")?;
    writeln!(temp_file, "# Lines starting with # are ignored")?;

    temp_file.flush()?;

    let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    let status = Command::new(&editor).arg(temp_file.path()).status()?;

    if !status.success() {
        return Err(format!("Editor '{}' exited with an error. Make sure your EDITOR environment variable is set correctly.", editor).into());
    }

    let edited_content = fs::read_to_string(temp_file.path())?;

    let new_items = parse_edit_commands(&edited_content, &todo_list.items)?;
    todo_list.items = new_items;

    todo_list.save_to_file()?;

    println!("Todo list updated!");

    Ok(())
}

/// Parse edit commands and return new list of todos
fn parse_edit_commands(
    content: &str,
    original_items: &[TodoItem],
) -> Result<Vec<TodoItem>, Box<dyn Error>> {
    let mut new_items = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.splitn(3, ' ').collect();

        if parts.len() < 2 {
            continue; // Skip malformed lines
        }

        let command = parts[0];
        let index_str = parts[1];

        let index: usize = match index_str.parse::<usize>() {
            Ok(i) if i > 0 && i <= original_items.len() => i - 1, // Convert to 0-based
            _ => continue,                                        // Skip invalid indices
        };

        match command {
            "pick" | "p" => {
                if let Some(item) = original_items.get(index) {
                    new_items.push(item.clone());
                }
            }
            "drop" | "d" => {}
            _ => {
                if let Some(item) = original_items.get(index) {
                    new_items.push(item.clone());
                }
            }
        }
    }

    Ok(new_items)
}

/// Done command -- Mark todos as completed using hash prefixes
fn done(hash: &[String]) -> Result<(), Box<dyn Error>> {
    let mut todo_list = TodoList::load_from_file()?;

    if todo_list.items.is_empty() {
        return Err("No todos available to mark as done".into());
    }

    let mut completed_count = 0;
    let mut errors = Vec::new();

    for prefix in hash {
        if prefix.trim().is_empty() {
            errors.push("Empty hash prefix provided".to_string());
            continue;
        }

        match todo_list.mark_done(prefix) {
            Ok(_) => {
                completed_count += 1;
                println!("Marked todo [{}] as done ✓", prefix);
            }
            Err(MarkDoneError::NotFound(msg)) => {
                errors.push(msg);
            }
            Err(MarkDoneError::AlreadyCompleted(msg)) => {
                errors.push(msg);
            }
            Err(MarkDoneError::MultipleMatches(matched_prefix, matches)) => {
                println!(
                    "Multiple todos found matching '{}', please be more specific:",
                    matched_prefix
                );
                for (hash, desc) in matches {
                    println!("[{}] {}", hash, desc);
                }
            }
        }
    }

    if completed_count > 0 {
        todo_list.save_to_file()?;
    }

    if !errors.is_empty() {
        for error in &errors {
            eprintln!("{}", error);
        }
        if completed_count == 0 {
            return Err("No todos were marked as done".into());
        }
    }

    if completed_count > 0 {
        println!("Successfully marked {} todo(s) as done!", completed_count);
    }

    Ok(())
}

#[cfg(test)]
mod test {

    #[test]
    pub fn add_cmd() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
