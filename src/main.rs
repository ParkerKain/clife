use std::fs::create_dir_all;
use std::fs::read_dir;
use std::fs::remove_file;
use std::fs::File;
use std::io::stdin;
use std::path::PathBuf;
use std::process::exit;

/// Represents all settings the user can set
struct Config {
    /// Where everything will be stored locally
    root_dir: PathBuf,
}

/// Represents a single note files
#[derive(Debug)]
struct Note {
    full_path: PathBuf,
    trunc_path: PathBuf,
}

#[derive(Debug)]
enum Action {
    CreateNote,
    Delete,
    CreateProject,
}

/// Returns if the root dir exists already
///
/// # Arguments
///
/// * `config` - a reference to a config object
fn detect_root_folder(config: &Config) -> bool {
    let exists = config.root_dir.try_exists();
    if exists.is_ok() {
        return exists.unwrap();
    } else {
        panic!("Failed to parse root dir {}", config.root_dir.display());
    }
}

/// Creates a root folder to store things in
///
/// # Arguments
///
/// * `config` - a reference to a config object
fn create_root_folder(config: &Config) {
    _ = create_dir_all(&config.root_dir);
    println!("{} directory created!", config.root_dir.display());
}

/// Creates the core notes vector from the root directory
///
/// # Arguments
///
/// * `config` - a reference to a config object
fn create_note_objects(config: &Config) -> Vec<Note> {
    let mut notes: Vec<Note> = Vec::new();
    _get_dir_notes(&config.root_dir, &mut notes, &config.root_dir);
    return notes;
}

/// Creates notes from the base directory - recurses through directories
///
/// # Arguments
///
/// * `base` - a reference to the base directory to search
/// * `notes` - The current state of a vector of notes to append to
/// * `root_dir` - the overall root_dir of the run
fn _get_dir_notes(base: &PathBuf, notes: &mut Vec<Note>, root_dir: &PathBuf) {
    let contents = read_dir(base).unwrap();
    for curr in contents {
        let curr_file = curr.expect("Failed to read");
        let curr_path = curr_file.path();
        if curr_path.is_dir() {
            _get_dir_notes(&curr_path, notes, root_dir);
        } else {
            let trunc_path = curr_path
                .strip_prefix(root_dir.to_path_buf())
                .unwrap()
                .to_path_buf();
            let curr_note = Note {
                full_path: curr_path,
                trunc_path,
            };
            notes.push(curr_note)
        }
    }
}

/// Prompts the user for the action they want to take
fn prompt_for_action() -> Action {
    let mut input = String::new();
    while !["c", "d", "p"].contains(&input.trim()) {
        input = String::new();
        println!("\nWhat action would you like to take?");
        println!("Options are ... \n\t - (c)reate note\n\t - (d)elete\n\t - create (p)roject");
        stdin().read_line(&mut input).expect("Failed to read line");
    }

    if input.trim() == "c" {
        return Action::CreateNote;
    } else if input.trim() == "d" {
        return Action::Delete;
    } else if input.trim() == "p" {
        return Action::CreateProject;
    } else {
        panic!("Unknown input");
    }
}

/// Creates a new note markdown file
///
/// # Arguments
///
/// * `config` - the config file that controls the run
/// * `note_suffix` - the number of the note to start with as a suffix
fn create_new_note(config: &Config, mut note_suffix: usize) -> PathBuf {
    let mut note_created = false;
    let mut note_path = PathBuf::from(&config.root_dir);
    while !note_created {
        note_path = PathBuf::from(&config.root_dir);
        let mut note_name = String::from("new_note_");
        note_name.push_str(&note_suffix.to_string());
        note_name.push_str(".md");
        note_path.push(&note_name);
        if note_path.exists() {
            println!("{} already exists, trying again ...", note_name);
            note_suffix += 1;
            continue;
        }
        let _ = File::create(&note_path);
        println!("New note created: {}", note_name);
        note_created = true;
    }
    return note_path;
}

/// Prompts the user for a note to take action on
///
/// # Arguments
///
/// * `notes` - a reference to the notes vector
/// * `action` - an action to take, only used to prompt the user
fn prompt_for_note(notes: &Vec<Note>, action: String) -> PathBuf {
    let mut input = String::new();
    let mut valid_input_passed: bool = false;
    while !valid_input_passed {
        input = String::new();
        println!("\nWhat file would you like to {}?", action);
        println!("Options are ... ");
        for note in notes {
            println!("- {:?}", note.trunc_path.as_os_str());
        }
        stdin().read_line(&mut input).expect("Failed to read line");
        if notes
            .iter()
            .any(|e| e.trunc_path.to_str() == Some(&input.as_str().trim()))
        {
            valid_input_passed = true;
        }
    }

    return PathBuf::from(input.trim());
}

/// Confirms with the user that they want a file to be deleted
///
/// # Arguments
///
/// * `path` - the potential file path to delete
fn confirm_delete(path: &PathBuf) {
    let mut input = String::new();
    while !["n", "y"].contains(&input.trim()) {
        input = String::new();
        println!("\nAre you sure you want to delete {}?", path.display());
        println!("Options are ... \n\t- (y)es\n\t- (n)o");
        stdin().read_line(&mut input).expect("Failed to read line");
    }

    if &input.trim() == &"n" {
        println!("Cancelling ...");
        exit(0);
    }
}

/// Deletes the passed PathBuf
///
/// # Arguments:
///
/// * `full_path` - the file path to delete
fn delete(full_path: PathBuf) -> bool {
    println!("Deleting note {} ...", full_path.display());
    let result = remove_file(full_path);
    match result {
        Ok(()) => {
            println!("File successfully deleted");
        }
        Err(e) => {
            panic!("Failed to delete file: {:?}", e);
        }
    }

    return true;
}

/// Prompts the user for a valid project name
fn prompt_for_project_name() -> String {
    let mut input = String::new();
    let mut valid_input = false;
    while !valid_input {
        input = String::new();
        println!("\nWhat would you like to name this project?");
        stdin().read_line(&mut input).expect("Failed to read line");

        // Ensure the input is a valid directory name
        valid_input = validate_project_name(&input);
        if !valid_input {
            println!(
                "Potential project name {} contains invalid characters",
                input
            );
            println!("May only use alphanumerics, '_', and '.'");
        }
    }
    return String::from(input.trim());
}

/// Ensures the passed project_name is a valid directory name
///
/// # Arguments 
///
/// * project_name - a reference to the project_name
fn validate_project_name(project_name: &String) -> bool {
    if project_name.trim().len() == 0 {
        return false;
    }

    // Ensure the input is a valid directory name
    let valid_input = project_name
        .trim()
        .chars()
        .all(|c| char::is_alphanumeric(c) || ['_', '.'].contains(&c));
    return valid_input;
}

fn main() {
    println!("Welcome to clife!");

    let config = Config {
        root_dir: PathBuf::from("/home/parker/.clife"),
    };

    if !detect_root_folder(&config) {
        println!("No clife folder detected at {}", config.root_dir.display());
        create_root_folder(&config);
    }

    let notes = create_note_objects(&config);
    println!("Found {} notes", notes.len());

    let action = prompt_for_action();

    match action {
        Action::CreateNote => {
            let note_path = create_new_note(&config, notes.len() + 1);
            let _ = std::process::Command::new("nvim")
                .arg(&note_path.into_os_string())
                .status();
        }
        Action::Delete => {
            let note_path = prompt_for_note(&notes, String::from("delete"));
            confirm_delete(&note_path);
            let mut full_path = config.root_dir.clone();
            full_path.push(&note_path);
            delete(full_path);
        }
        Action::CreateProject => {
            let project_name = prompt_for_project_name();
        }
        _ => {
            println!("Unknown action")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_root_folder_exists() {
        let config = Config {
            root_dir: PathBuf::from("/home"),
        };
        let result: bool = detect_root_folder(&config);
        assert_eq!(result, true)
    }

    #[test]
    fn test_detect_root_folder_not_exists() {
        let config = Config {
            root_dir: PathBuf::from("~/nonsense_folder_ntuyfwntw/"),
        };
        let result: bool = detect_root_folder(&config);
        assert_eq!(result, false)
    }

    #[test]
    fn test_create_note_objects() {
        let config = Config {
            root_dir: PathBuf::from(
                "/home/parker/Documents/projects/clife/clife/test_data/.clife/",
            ),
        };
        let result: Vec<Note> = create_note_objects(&config);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_valid_project_name() {
        let valid_names = ["test", "test_1", "my.project", ".HELLO.P_Arker_", "   hello   "];

        for name in valid_names {
            assert_eq!(validate_project_name(&String::from(name)), true);
        }
    }

    #[test]
    fn test_invalid_project_name() {
        let invalid_names = ["hello parker", "&parker", "_hello_("];

        for name in invalid_names {
            assert_eq!(validate_project_name(&String::from(name)), false);
        }
    }
}
