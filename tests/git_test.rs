use rusty_commit::git;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn init_test_repo() -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();

    // Initialize a git repo
    Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to init git repo");

    // Configure git user for commits
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git email");

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to set git name");

    temp_dir
}

fn init_test_repo_with_commit() -> tempfile::TempDir {
    let temp_dir = init_test_repo();

    // Create initial commit to establish main branch
    fs::write(temp_dir.path().join(".gitignore"), "").expect("Failed to create .gitignore");
    Command::new("git")
        .args(["add", ".gitignore"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to add .gitignore");

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to create initial commit");

    temp_dir
}

#[test]
fn test_assert_git_repo() {
    let temp_dir = init_test_repo();
    let original_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Should succeed in a git repo
    assert!(git::assert_git_repo().is_ok());

    // Should fail outside a git repo
    let non_git_dir = tempdir().unwrap();
    std::env::set_current_dir(non_git_dir.path()).unwrap();
    assert!(git::assert_git_repo().is_err());

    // Restore original directory
    std::env::set_current_dir(&original_cwd).unwrap();
    drop(temp_dir);
    drop(non_git_dir);
}

#[test]
fn test_get_changed_files() {
    let temp_dir = init_test_repo_with_commit();
    let original_cwd = std::env::current_dir().unwrap();

    // Change directory and ensure temp_dir stays alive
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // No changes initially (since we have a clean initial commit)
    let files = git::get_changed_files().unwrap();
    assert_eq!(files.len(), 0);

    // Create a new file
    fs::write(temp_dir.path().join("test.txt"), "hello").unwrap();

    let files = git::get_changed_files().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0], "test.txt");

    // Restore original directory
    std::env::set_current_dir(&original_cwd).unwrap();

    // Keep temp_dir alive until end of test
    drop(temp_dir);
}

#[test]
fn test_get_staged_files() {
    let temp_dir = init_test_repo_with_commit();
    let original_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Create and stage a file
    fs::write(temp_dir.path().join("staged.txt"), "content").unwrap();
    Command::new("git")
        .args(["add", "staged.txt"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let staged = git::get_staged_files().unwrap();
    assert_eq!(staged.len(), 1);
    assert_eq!(staged[0], "staged.txt");

    // Create an unstaged file
    fs::write(temp_dir.path().join("unstaged.txt"), "content").unwrap();

    let staged = git::get_staged_files().unwrap();
    assert_eq!(staged.len(), 1); // Still only one staged file

    // Restore original directory
    std::env::set_current_dir(&original_cwd).unwrap();
    drop(temp_dir);
}

#[test]
fn test_stage_files() {
    let temp_dir = init_test_repo_with_commit();
    let original_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Create multiple files
    fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();
    fs::write(temp_dir.path().join("file3.txt"), "content3").unwrap();

    // Stage specific files
    git::stage_files(&["file1.txt".to_string(), "file3.txt".to_string()]).unwrap();

    let staged = git::get_staged_files().unwrap();
    assert_eq!(staged.len(), 2);
    assert!(staged.contains(&"file1.txt".to_string()));
    assert!(staged.contains(&"file3.txt".to_string()));
    assert!(!staged.contains(&"file2.txt".to_string()));

    // Restore original directory
    std::env::set_current_dir(&original_cwd).unwrap();
    drop(temp_dir);
}

#[test]
fn test_get_staged_diff() {
    let temp_dir = init_test_repo_with_commit();
    let original_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Create and stage changes
    fs::write(temp_dir.path().join("new_file.txt"), "new content").unwrap();
    fs::write(temp_dir.path().join("initial.txt"), "initial content").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let diff = git::get_staged_diff().unwrap();
    assert!(diff.contains("new_file.txt"));
    assert!(diff.contains("new content"));
    assert!(diff.contains("initial.txt"));
    assert!(diff.contains("initial content"));

    // Restore original directory
    std::env::set_current_dir(&original_cwd).unwrap();
    drop(temp_dir);
}

#[test]
fn test_get_repo_root() {
    let temp_dir = init_test_repo();
    let original_cwd = std::env::current_dir().unwrap();
    let sub_dir = temp_dir.path().join("subdir");
    fs::create_dir(&sub_dir).unwrap();

    // Test from root
    std::env::set_current_dir(temp_dir.path()).unwrap();
    let root = git::get_repo_root().unwrap();
    // Use canonicalize to resolve symlinks for comparison
    let expected = temp_dir.path().canonicalize().unwrap();
    let actual = std::path::PathBuf::from(&root).canonicalize().unwrap();
    assert_eq!(actual, expected);

    // Test from subdirectory
    std::env::set_current_dir(&sub_dir).unwrap();
    let root = git::get_repo_root().unwrap();
    let actual = std::path::PathBuf::from(&root).canonicalize().unwrap();
    assert_eq!(actual, expected);

    // Restore original directory
    std::env::set_current_dir(&original_cwd).unwrap();
    drop(temp_dir);
}
