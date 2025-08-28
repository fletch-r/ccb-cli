use std::{env, fs, io::Write, process};
use std::path::PathBuf;
use dialoguer::MultiSelect;
use git2::{Repository, Signature, Status};
use git2_credentials::CredentialHandler;
/*
Creates to use
    - Clap
        - Command Line Parser - helps with reducing if statements
        - To be used in the future probably, right now the cli won't have any args
    - git2
        - Shortcut to read `git status`
        - Should use now
    - console
        - Helps to style console output
        - Maybe will use
*/

/// Adds an alias for `ccb` to the user's shell profile so that typing `ccb` runs the binary.
/// This function checks the user's shell and appends the alias to the appropriate profile file.
/// It only adds the alias if it doesn't already exist.
pub fn add_ccb_to_path_once() {
    // Determine the user's home directory
    let home_dir = match env::var("HOME") {
        Ok(path) => PathBuf::from(path),
        Err(_) => {
            eprintln!("Could not determine home directory.");
            return;
        }
    };

    // Determine the shell and profile file
    let shell = env::var("SHELL").unwrap_or_default();
    let profile_file = if shell.contains("zsh") {
        home_dir.join(".zshrc")
    } else if shell.contains("bash") {
        home_dir.join(".bashrc")
    } else if shell.contains("fish") {
        home_dir.join(".config/fish/config.fish")
    } else {
        // Default to .bashrc if unknown
        home_dir.join(".bashrc")
    };

    // Use the absolute path to the ccb binary
    let ccb_binary_path = "/Users/andrewfletcher/RustroverProjects/ccb-cli/target/debug/ccb";
    let alias_line = format!("alias ccb=\"{}\"", ccb_binary_path);

    // Check if the alias already exists
    if let Ok(contents) = fs::read_to_string(&profile_file) {
        if contents.contains("alias ccb=") {
            // Alias already exists, do nothing
            return;
        }
    }

    // Append the alias to the profile file
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&profile_file) {
        if let Err(e) = writeln!(file, "\n{}", alias_line) {
            eprintln!("Failed to write alias to {:?}: {}", profile_file, e);
        } else {
            println!("Added alias for `ccb` to {:?}", profile_file);
            println!("You may need to restart your terminal or run `source {:?}` to use the alias.", profile_file);
        }
    } else {
        eprintln!("Could not open profile file: {:?}", profile_file);
    }
}



fn get_statuses() -> (Vec<String>, Vec<bool>) {
    let repo = Repository::open(".").unwrap();
    let statuses = repo.statuses(None).unwrap();

    if statuses.is_empty() {
        println!("{}", console::style("✔ working tree clean").green());
        process::exit(1);
    }

    let mut items = vec![];
    let mut default_checked = vec![];

    for entry in statuses.iter() {
        let path = entry
            .head_to_index()
            .and_then(|d| d.new_file().path())
            .or_else(|| entry.index_to_workdir().and_then(|d| d.new_file().path()))
            .or_else(|| entry.index_to_workdir().and_then(|d| d.old_file().path()))
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<unknown>".into());

        let s = entry.status();

        let b = is_staged(s);

        if s.contains(Status::IGNORED) {
            continue;
        }
        items.push(path);
        default_checked.push(b);
    }

    (items, default_checked)
}

pub fn choose_files() -> Vec<String> {
    let (items, defaults) = get_statuses();

    let selection = MultiSelect::new()
        .with_prompt("Choose files to stage")
        .items(&items)
        .defaults(&defaults)
        .interact()
        .unwrap();

    let mut paths: Vec<String> = vec![];

    for i in selection {
        println!("{}", items[i]);
        paths.push(items[i].clone());
    }

    paths
}

fn is_staged(s: Status) -> bool {
    // Index (staged)
    // If it is staged, it should be pre-selected when running ccb

    let index = vec![
        Status::INDEX_NEW,
        Status::INDEX_MODIFIED,
        Status::INDEX_DELETED,
        Status::INDEX_RENAMED,
        Status::INDEX_TYPECHANGE,
        Status::CONFLICTED,
    ];

    for status in index {
        if s.contains(status) {
            return true;
        }
    }

    if s.contains(Status::WT_NEW) && s.contains(Status::INDEX_NEW) {
        true
    } else if s.contains(Status::WT_MODIFIED) && s.contains(Status::INDEX_MODIFIED) {
        return true;
    } else if s.contains(Status::WT_DELETED) && s.contains(Status::INDEX_DELETED) {
        return true;
    } else if s.contains(Status::WT_RENAMED) && s.contains(Status::INDEX_RENAMED) {
        return true;
    } else if s.contains(Status::WT_TYPECHANGE) && s.contains(Status::INDEX_TYPECHANGE) {
        return true;
    } else if s.contains(Status::WT_NEW) && !s.intersects(Status::INDEX_NEW) {
        return true;
    } else {
        let worktree = vec![
            Status::WT_NEW,
            Status::WT_MODIFIED,
            Status::WT_DELETED,
            Status::WT_RENAMED,
            Status::WT_TYPECHANGE,
            Status::CONFLICTED
        ];

        for status in worktree {
            if s.contains(status) {
                return false;
            }
        }

        return false;
    }
}

pub fn get_commit_message() -> String {
    let message: String = dialoguer::Input::new()
        .with_prompt("Write a commit message")
        .interact_text()
        .unwrap();

    message
}

pub fn commit(repo: &Repository, message: &String) {
    let mut index = repo.index().unwrap();
    let tree = repo
        .find_tree(index.write_tree().unwrap())
        .unwrap();
    let author = Signature::now("fletch-r", "ath3ris@proton.me").unwrap();

    let parent_commit = repo.head().unwrap().peel_to_commit().unwrap();

    let update_ref = if repo.head().is_ok() {
        Some("HEAD")
    } else {
        None // no HEAD = first commit
    };

    repo.commit(update_ref, &author, &author, &message, &tree, &[&parent_commit]).unwrap();
}

pub fn push_confirm() -> bool {
    let yes: String = dialoguer::Input::new()
        .with_prompt("Push now?")
        .interact_text()
        .unwrap();

    if yes == "yes" || yes == "y" {
        true
    } else {
        false
    }
}

pub fn git_commit_push(repo: &Repository, my_branch: String) {
    let mut origin = repo.find_remote("origin").unwrap();
    // Setup authentication callbacks for push
    let git_config_for_push = git2::Config::open_default().unwrap();
    let mut credential_handler_for_push = CredentialHandler::new(git_config_for_push);
    let mut push_callbacks = git2::RemoteCallbacks::new();
    push_callbacks.credentials(move |url, username, allowed| {
        credential_handler_for_push.try_next_credential(url, username, allowed)
    });

    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(push_callbacks);

    // Push the current HEAD to the specified remote branch
    origin
        .push(&["HEAD:refs/heads/".to_owned() + my_branch.as_str()], Some(&mut push_options))
        .unwrap();
}

pub fn git_add_selected(repo: &Repository, paths: &Vec<String>) {
    let mut index = repo.index().unwrap();
    index.add_all(paths, git2::IndexAddOption::DEFAULT, None).unwrap();
    index.write().unwrap();
}

pub fn get_current_branch_name(repo: &Repository) -> Option<String> {
    match repo.head() {
        Ok(head) => {
            if head.is_branch() {
                head.shorthand().map(|s| s.to_string())
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use git2::Status;
    use crate::get_statuses;
    use crate::is_staged;

    // === Set-up

    // Download CCB

    // Make a config file

    // Read a config file

    /*
        Config file:
        REFERENCE_REGEX = (?!.*\/)([^\\d]*)(\\d+) // Gets the ticket/issue number
        COMMIT_TEMPLATE = "<type><scope>: <reference> - <description>\n\n<body>\n\n<footer>"
    */

    // === Functionality

    // Choose files to commit

    #[test]
    fn statuses_has_length() {
        let (items, _) = get_statuses();
        assert!(items.len() > 0);
    }

    // Already staged files are checked by default

    #[test]
    fn status_returns_true() {
        let value = is_staged(Status::INDEX_MODIFIED);
        assert!(value);
    }

    #[test]
    fn status_returns_false() {
        let value = is_staged(Status::WT_MODIFIED);
        assert_eq!(value, false);
    }

    // Choose type

    // Enter scope

    // Get reference

    // Enter description

    // Enter body

    // Enter footer
}

// Add command to PATH

// Ask what files you would like to add: "git add <file_path>"

// Prompt for a commit message

// Use regex to get branch ticket/issue id
