use git2::{Repository};
use ccb_cli::{choose_files, git_add_selected, get_commit_message, commit, push_confirm, git_commit_push };

fn main() {
    let repo = Repository::open(".").unwrap();

    let paths = choose_files();

    git_add_selected(&repo, &paths);

    println!("Chosen paths to commit - {:#?}", paths);

    let message = get_commit_message();

    commit(&repo, message);

    if push_confirm() {
        git_commit_push(&repo, "test");
    }
}
