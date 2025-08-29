use git2::{Repository};
use ccb_cli::{
    choose_files,
    git_add_selected,
    get_commit_message,
    commit,
    push_confirm,
    git_commit_push,
    get_current_branch_name,
    add_ccb_to_path_once
};

fn main() {
    add_ccb_to_path_once();

    let repo = Repository::open(".").unwrap();

    let paths = choose_files();

    git_add_selected(&repo, &paths);

    let message = get_commit_message(&repo);

    if let Err(error) = commit(&repo, &message) {
        eprintln!("Commit failed: {}", error);
        std::process::exit(1);
    }

    println!("Chosen paths to commit - {:#?}", paths);
    println!("Commit message: {}", message);

    let branch_name = get_current_branch_name(&repo).unwrap_or_else(|| panic!("Could not find current branch name"));

    if push_confirm() {
        git_commit_push(&repo, branch_name);
    }
}
