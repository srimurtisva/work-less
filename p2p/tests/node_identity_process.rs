use std::fs;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use tempfile::TempDir;

#[test]
fn two_processes_only_one_gets_persistent_identity() {
    // When this executable is started with NODE_IDENTITY_CHILD,
    // the test harness will execute `child_identity_lock_test`.
    if std::env::var_os("NODE_IDENTITY_CHILD").is_some() {
        unreachable!("The child process should execute the dedicated child test");
    }

    let temp_dir = TempDir::new().unwrap();

    let first_result = temp_dir.path().join("first.result");
    let second_result = temp_dir.path().join("second.result");

    let current_exe = std::env::current_exe().unwrap();

    // Start the first child process.
    let mut first = Command::new(&current_exe)
        .arg("--exact")
        .arg("child_identity_lock_test")
        .arg("--nocapture")
        .env("NODE_IDENTITY_CHILD", "1")
        .env(
            "NODE_IDENTITY_DATA_DIR",
            temp_dir.path(),
        )
        .env(
            "NODE_IDENTITY_RESULT_FILE",
            &first_result,
        )
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    // Wait until the first child has acquired the persistent identity.
    wait_for_file(&first_result);

    let first_result = fs::read_to_string(&first_result).unwrap();

    assert_eq!(
        first_result.lines().next(),
        Some("persistent=true")
    );

    // Start the second child while the first child is still alive
    // and therefore still owns the identity lock.
    let mut second = Command::new(&current_exe)
        .arg("--exact")
        .arg("child_identity_lock_test")
        .arg("--nocapture")
        .env("NODE_IDENTITY_CHILD", "1")
        .env(
            "NODE_IDENTITY_DATA_DIR",
            temp_dir.path(),
        )
        .env(
            "NODE_IDENTITY_RESULT_FILE",
            &second_result,
        )
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    wait_for_file(&second_result);

    let second_result = fs::read_to_string(&second_result).unwrap();

    assert_eq!(
        second_result.lines().next(),
        Some("persistent=false")
    );

    let first_id = first_result
        .lines()
        .find_map(|line| line.strip_prefix("id="))
        .expect("first process did not report its identity");

    let second_id = second_result
        .lines()
        .find_map(|line| line.strip_prefix("id="))
        .expect("second process did not report its identity");

    // The two processes must have different endpoint identities.
    assert_ne!(first_id, second_id);

    // Kill the first process while it still owns the lock.
    // This verifies that the lock is released even after an abrupt
    // process termination.
    first.kill().unwrap();
    first.wait().unwrap();

    // The second process has an ephemeral identity and does not need
    // to be kept alive for this test.
    second.kill().unwrap();
    second.wait().unwrap();
}

#[test]
fn child_identity_lock_test() {
    // This test is only intended to be executed as a child process
    // of `two_processes_only_one_gets_persistent_identity`.
    if std::env::var_os("NODE_IDENTITY_CHILD").is_none() {
        return;
    }

    run_child_identity_lock_test();
}

fn run_child_identity_lock_test() {
    let data_dir = std::env::var_os("NODE_IDENTITY_DATA_DIR")
        .expect("NODE_IDENTITY_DATA_DIR is not set");

    let result_file = std::env::var_os("NODE_IDENTITY_RESULT_FILE")
        .expect("NODE_IDENTITY_RESULT_FILE is not set");

    let mut manager =
        p2p::node_identity::NodeIdentityManager::new(data_dir);

    let identity = manager.acquire_identity();

    let result = format!(
        "persistent={}\nid={}\n",
        identity.owns_persistent_identity,
        identity.key.public(),
    );

    fs::write(&result_file, result)
        .expect("failed to write child process result");

    // Keep the manager alive so that it continues to own the lock.
    // The parent process will terminate us after verifying the result.
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

fn wait_for_file(path: &std::path::Path) {
    let deadline = Instant::now() + Duration::from_secs(10);

    while !path.exists() {
        if Instant::now() >= deadline {
            panic!(
                "Timed out waiting for child process result: {:?}",
                path
            );
        }

        thread::sleep(Duration::from_millis(10));
    }
}

