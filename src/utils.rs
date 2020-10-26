use fs_extra;
use fs_extra::dir::CopyOptions;
use std::fs;
use std::process::Command;

use crate::AFLRun;

pub fn mv(from: String, to: String) {
    copy(from.clone(), to.clone());
    std::fs::remove_dir_all(&from)
        .expect(format!("Could not remove '{}' in utils::mv", from).as_str());
}

pub fn copy(from: String, to: String) {
    //let options = CopyOptions::new();
    Command::new("cp")
        .args(&[String::from("-r"), from.clone(), to.clone()])
        .spawn()
        .expect(format!("[!] Could not copy from {} to {}", from, to).as_str())
        .wait()
        .expect("[-] Failed waiting for copy.");
}

pub fn copy_ignore(from: String, to: String) {
    let options = CopyOptions::new();
    match fs_extra::dir::copy(&from, &to, &options) {
        _ => {}
    }
}

#[allow(dead_code)]
pub fn rm(dir: String) {
    Command::new("rm")
        .args(&["-rf", dir.clone().as_str()])
        .spawn()
        .expect("[!] Could not start removing dir/file")
        .wait()
        .expect(format!("[!] Removing dir/file {} failed.", dir).as_str());
}

pub fn copy_snapshot_base(base_state: &String, state_path: &String) -> () {
    // copy old snapshot folder for criu
    let old_snapshot = format!("./saved-states/{}/snapshot", base_state);
    let new_snapshot = format!("./active-state/{}/", state_path);

    // Check fs_extra docs for different copy options
    let options = CopyOptions::new();
    fs_extra::dir::copy(old_snapshot, new_snapshot, &options)
        .expect("[!] Could not copy snapshot dir from previous state");

    // copy old pipes file so restore.sh knows which pipes are open
    let old_pipes = format!("./saved-states/{}/pipes", base_state);
    let new_pipes = format!("./active-state/{}/pipes", state_path);
    fs::copy(old_pipes, new_pipes)
        .expect("[!] Could not copy old pipes file to new state-dir");
}

pub fn create_restore_sh(afl: &AFLRun) {
    let _ = Command::new("python3")
        .args(&[
            "create_restore.py".to_string(),
            afl.base_state.to_string(),
            afl.active_dir.to_string(),
        ])
        .spawn()
        .expect("[!] Could not spawn create_restore.py")
        .wait()
        .expect("[!] Could not create restore.sh with python");
}

/// Create the next iteration from a given state directory. If inc_server is set
/// we will increment the state for the server from fitm-cXsY to fitm-cXsY+1.
/// Otherwise we will increment the state for the client from fitm-cXsY to
/// fitm-cX+1sY
pub fn next_state_path(
    state_path: (u32, u32),
    cur_is_server: bool,
) -> (u32, u32) {
    // If inc_server increment the server state else increment the client state
    if cur_is_server {
        ((state_path.0) + 1, state_path.1)
    } else {
        (state_path.0, (state_path.1) + 1)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils;
    use std::fs;
    use std::path::Path;

    fn setup(
        root_folder: &String,
        from_path: &String,
        from_content_path: &String,
        content: &str,
    ) {
        // setup - require user interaction so we don't delete anything by
        // default Creates necessary files/folders under /tmp
        fs_extra::dir::create(root_folder, false).expect("rust_unittest folder already exists, please remove to make this test run");
        fs_extra::dir::create_all(from_path, true)
            .expect("Could not create test folder");
        fs::write(from_content_path, content)
            .expect("Could not write to 'from' content.txt");
    }

    fn teardown(root_folder: &String) {
        // Remove all files created during the test
        std::fs::remove_dir_all(root_folder)
            .expect("Could not remove rust_unittest folder");
    }

    fn paths_exist(root_folder: &String, to_content_path: &String) -> bool {
        // Returns true if all three of the given paths exists
        let bool_1 = Path::new(&format!("{}/foo", root_folder)).exists();
        let bool_2 = Path::new(&format!("{}/bar", root_folder)).exists();
        let bool_3 = Path::new(to_content_path).exists();
        bool_1 && bool_2 && bool_3
    }

    fn check_is_dir(to_path: &String) -> bool {
        // Returns true if the given path points to a directory
        let metadata =
            fs::metadata(to_path).expect("Could not find copy 'to' folder");
        metadata.file_type().is_dir()
    }

    #[test]
    fn test_copy() {
        // Test whether utils::copy() copies recursively to a given path,
        // using the original folders name as target name
        let root_folder = String::from("/tmp/rust_unittest");
        let from_path = format!("{}/foo/bar", root_folder);
        let to_path = format!("{}", root_folder);
        let from_content_path = format!("{}/foo/bar/content.txt", root_folder);
        let to_content_path = format!("{}/bar/content.txt", root_folder);
        let content = "A simple string.";

        setup(&root_folder, &from_path, &from_content_path, content);

        // tested function
        utils::copy(from_path.clone(), root_folder.clone());

        // Check that the 'from' path does not exist anymore, but the 'to' path
        // does
        assert_eq!(Path::new(&from_path).exists(), true);
        assert!(paths_exist(&root_folder, &to_content_path));

        // Check 'to' path is still a directory
        assert!(check_is_dir(&to_path));

        // Check that the content of the copied folder still exists
        let result_content = std::fs::read_to_string(to_content_path)
            .expect("Could not read from expected content.txt");

        assert_eq!(result_content, "A simple string.");

        // teardown
        teardown(&root_folder);
    }

    #[test]
    fn test_mv() {
        // Check that utils::mv moves a folder to a new destination
        let root_folder = String::from("/tmp/rust_unittest");
        let from_path = format!("{}/foo/bar", root_folder);
        let to_path = format!("{}", root_folder);
        let from_content_path = format!("{}/foo/bar/content.txt", root_folder);
        let to_content_path = format!("{}/bar/content.txt", root_folder);
        let content = "A simple string.";

        setup(&root_folder, &from_path, &from_content_path, content);

        // tested function
        utils::mv(from_path.clone(), to_path.clone());

        // Check that the 'from' path does not exist anymore, but the 'to' path
        // does
        assert_eq!(Path::new(&from_path).exists(), false);
        assert!(paths_exist(&root_folder, &to_content_path));

        // Check 'to' path is still a directory
        assert!(check_is_dir(&to_path));

        // Check that the content of the copied folder still exists
        let result_content = std::fs::read_to_string(to_content_path)
            .expect("Could not read from expected content.txt");

        assert_eq!(result_content, "A simple string.");

        // teardown
        teardown(&root_folder);
    }

    #[test]
    fn test_remove_dir_all() {
        let root_folder = String::from("/tmp/rust_unittest");
        let path = format!("{}/foo/bar", root_folder);
        let content_path = format!("{}/foo/bar/content.txt", root_folder);
        let content = "A simple string.";

        setup(&root_folder, &path, &content_path, content);

        let foo_path = format!("{}/foo", root_folder);

        // tested function
        std::fs::remove_dir_all(&foo_path)
            .expect("Tested remove_dir_all failed");

        assert_eq!(Path::new(&foo_path).exists(), false);

        teardown(&root_folder);
    }
}
