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

fn get_statuses() -> (Vec<String>, Vec<bool>) {
    let repo = Repository::open(".").unwrap();
    let statuses = repo.statuses(None).unwrap();

    if statuses.is_empty() {
        println!("{}", console::style("✔ working tree clean").green());
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
        Status::IGNORED,
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

pub fn commit(repo: &Repository, message: String) {
    let mut index = repo.index().unwrap();
    let tree = repo
        .find_tree(index.write_tree().unwrap())
        .unwrap();
    let author = Signature::now("fletchers", "ath3ris@proton.me").unwrap();

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

pub fn git_commit_push(repo: &Repository, my_branch: &str) {
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
        .push(&["HEAD:refs/heads/".to_owned() + my_branch], Some(&mut push_options))
        .unwrap();
}

pub fn git_add_selected(repo: &Repository, paths: &Vec<String>) {
    let mut index = repo.index().unwrap();
    index.add_all(paths, git2::IndexAddOption::DEFAULT, None).unwrap();
    index.write().unwrap();
}

#[cfg(test)]
mod tests {
    use git2::Status;
    use crate::get_statuses;
    use crate::is_staged;

    #[test]
    fn statuses_has_length() {
        let (items, _) = get_statuses();
        assert!(items.len() > 0);
    }

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
}

// Add command to PATH

// Ask what files you would like to add: "git add <file_path>"

// Prompt for a commit message

// Use regex to get branch ticket/issue id
