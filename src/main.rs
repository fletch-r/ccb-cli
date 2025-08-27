mod example;

use std::env;
use git2::{IndexAddOption, Repository, Signature, Branch};
use ccb_cli::choose_files;
use log::debug;

fn git_commit_push(repo: &Repository, my_branch: &str) {
    let mut index = repo.index().unwrap();
    let tree = repo
        .find_tree(index.write_tree().unwrap())
        .unwrap();
    let author = Signature::now("fletchers", "ath3ris@proton.me").unwrap();

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

fn git_add_all(repo: &Repository, paths: &Vec<String>) {
    let mut index = repo.index().unwrap();
    index
        .add_all(&["."], git2::IndexAddOption::DEFAULT, None)
        .unwrap();
    index.write().unwrap();
}

fn main() {
    let paths = choose_files();

    println!("Chosen paths to commit - {:#?}", paths);

    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(git_credentials_callback);

    let mut opts = git2::FetchOptions::new();
    opts.remote_callbacks(callbacks);
    opts.download_tags(git2::AutotagOption::All);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(opts);
    builder.branch("master");

    let repo = Repository::open(".").unwrap();

    git_add_all(&repo, &paths);

    git_commit_push(&repo, "master");

    // let current_branch = index.write_tree().unwrap();

    // git_commit_push(&repo, current_branch.to_string().as_str());
}

pub fn git_credentials_callback(
    _user: &str,
    _user_from_url: Option<&str>,
    _cred: git2::CredentialType,
) -> Result<git2::Cred, git2::Error> {
    let user = _user_from_url.unwrap_or("git");

    if _cred.contains(git2::CredentialType::USERNAME) {
        return git2::Cred::username(user);
    }

    match env::var("GPM_SSH_KEY") {
        Ok(k) => {
            debug!("authenticate with user {} and private key located in {}", user, k);
            git2::Cred::ssh_key(user, None, std::path::Path::new(&k), None)
        },
        _ => Err(git2::Error::from_str("unable to get private key from GPM_SSH_KEY")),
    }
}

// fn get_or_init_repo(cache: &std::path::Path, remote: &String) -> Result<git2::Repository, git2::Error> {
//     let data_url = match Url::parse(remote) {
//         Ok(data_url) => data_url,
//         Err(e) => panic!("failed to parse url: {}", e),
//     };
//     let path = cache.deref().join(data_url.host_str().unwrap()).join(&data_url.path()[1..]);
//
//     if path.exists() {
//         debug!("use existing repository already in cache {}", path.to_str().unwrap());
//         return git2::Repository::open(path);
//     }
//
//     let mut callbacks = git2::RemoteCallbacks::new();
//     callbacks.credentials(git_credentials_callback);
//
//     let mut opts = git2::FetchOptions::new();
//     opts.remote_callbacks(callbacks);
//     opts.download_tags(git2::AutotagOption::All);
//
//     let mut builder = git2::build::RepoBuilder::new();
//     builder.fetch_options(opts);
//     builder.branch("master");
//
//     debug!("start cloning repository {} in {}", remote, path.to_str().unwrap());
//
//     match builder.clone(remote, &path) {
//         Ok(r) => {
//             debug!("repository cloned");
//
//             Ok(r)
//         },
//         Err(e) => Err(e)
//     }
// }
