use git2::{Repository, Signature};

fn main(){

    let repo_url = "https://YOUR_GITHUB_TOKEN_GOES_HERE@github.com/User/repo.git";
    let repo_path = "/tmp/test/";

    let repo = match Repository::clone(repo_url, repo_path) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to clone: {}", e),
    };

    let my_branch = "test";

    let head = repo.head().unwrap();
    let oid = head.target().unwrap();
    let commit = repo.find_commit(oid).unwrap();
    let branch = repo.branch(
        my_branch,
        &commit,
        false,
    );
    let obj = repo.revparse_single(&("refs/heads/".to_owned() + my_branch)).unwrap();

    repo.checkout_tree(
        &obj,
        None
    );

    repo.set_head(&("refs/heads/".to_owned() + my_branch));

    // based on : https://github.com/rust-lang/git2-rs/issues/561
    let file_name = "test.txt";
    create_file(&repo_path, file_name);
    git_add_all(&repo);
    git_commit_push(&repo, my_branch);

}

fn create_file(repo_path:&str, file_name: &str) {
    let file_path= format!("{}{}", repo_path, file_name);
    std::fs::File::create(file_path).unwrap();
}

fn git_add_all(repo: &git2::Repository) {
    let mut index = repo.index().unwrap();
    index
        .add_all(&["."], git2::IndexAddOption::DEFAULT, None)
        .unwrap();
    index.write().unwrap();
}

fn git_commit_push(repo: &git2::Repository, my_branch: &str) {
    let mut index = repo.index().unwrap();
    let tree = repo
        .find_tree(index.write_tree().unwrap())
        .unwrap();
    let author = Signature::now("x", "x@x.xxx").unwrap();

    let mut update_ref = Some("HEAD");

    if let Ok(head) = repo.head() {
        update_ref = Some("HEAD");
    } else {
        update_ref = None; // no HEAD = first commit
    }

    let parent_commit = repo.head().unwrap().peel_to_commit().unwrap();

    let commit_oid = repo
        .commit(update_ref, &author, &author, "commit message", &tree, &[&parent_commit])
        .unwrap();

    let mut origin = repo.find_remote("origin").unwrap();
    origin.push(&["refs/heads/".to_owned() + my_branch], None).unwrap();
}