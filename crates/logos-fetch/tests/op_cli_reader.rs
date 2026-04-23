#![allow(unsafe_code)]

use std::env;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

use logos_fetch::{OpCliSecretRefReader, SecretRefReader};

fn create_mock_op(temp_dir: &TempDir, fail: bool) -> std::path::PathBuf {
    let (file_name, script) = if fail {
        if cfg!(windows) {
            (
                "op.bat",
                "@echo off\r\necho simulated error 1>&2\r\nexit /b 1",
            )
        } else {
            ("op", "#!/bin/sh\necho \"simulated error\" >&2\nexit 1")
        }
    } else {
        if cfg!(windows) {
            (
                "op.bat",
                "@echo off\r\nif \"%~1\"==\"read\" if \"%~2\"==\"op://test/val\" (\r\n  <nul set /p=\"exact_secret_value\"\r\n  exit /b 0\r\n)\r\necho unexpected args: %* 1>&2\r\nexit /b 1",
            )
        } else {
            (
                "op",
                "#!/bin/sh\nif [ \"$1\" = \"read\" ] && [ \"$2\" = \"op://test/val\" ]; then\n    printf \"exact_secret_value\"\n    exit 0\nfi\necho \"unexpected args: $@\" >&2\nexit 1",
            )
        }
    };

    let mock_op_path = temp_dir.path().join(file_name);
    fs::write(&mock_op_path, script).unwrap();

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&mock_op_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&mock_op_path, perms).unwrap();
    }

    mock_op_path
}

fn with_env_var<F>(key: &str, value: &str, f: F)
where
    F: FnOnce(),
{
    let old_val = env::var_os(key);
    unsafe {
        env::set_var(key, value);
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));

    if let Some(old) = old_val {
        unsafe {
            env::set_var(key, old);
        }
    } else {
        unsafe {
            env::remove_var(key);
        }
    }

    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}

#[test]
fn op_cli_reader_integration_tests() {
    let temp_dir = TempDir::new().unwrap();

    let mock_op_success = create_mock_op(&temp_dir, false);

    with_env_var(
        "LOGOS_FETCH_OP_BIN",
        mock_op_success.to_str().unwrap(),
        || {
            let reader = OpCliSecretRefReader::from_environment();
            let val = reader
                .read_secret_ref("op://test/val")
                .expect("read should succeed");
            assert_eq!(val, "exact_secret_value");
        },
    );

    let mock_op_failure = create_mock_op(&temp_dir, true);

    with_env_var(
        "LOGOS_FETCH_OP_BIN",
        mock_op_failure.to_str().unwrap(),
        || {
            let reader = OpCliSecretRefReader::from_environment();
            let err = reader
                .read_secret_ref("op://test/val")
                .expect_err("read should fail");
            assert!(
                err.to_string().contains("simulated error"),
                "unexpected error msg: {err}"
            );
        },
    );
}

// 👺 Havoc: Prove that `unsafe { env::set_var }` is fragile by attacking it concurrently
#[test]
fn havoc_test_unsafe_env_mutation_data_race() {
    use std::thread;

    // The target env var that the integration test relies on
    let target_key = "LOGOS_FETCH_OP_BIN";

    // We launch threads that continuously corrupt the environment variable while the integration test might be running.
    // If run together via `cargo test`, this guarantees a data race and process state corruption.
    let mut handles = vec![];

    for i in 0..10 {
        let handle = thread::spawn(move || {
            for j in 0..1000 {
                let val = format!("garbage_path_corruption_{}_{}", i, j);
                unsafe {
                    env::set_var(target_key, &val);
                }
                let _ = env::var(target_key).unwrap_or_default();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }
}
