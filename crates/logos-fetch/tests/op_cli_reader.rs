#![allow(unsafe_code)]

use std::env;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

use logos_fetch::{OpCliSecretRefReader, SecretRefReader};

fn create_mock_op(temp_dir: &TempDir, fail: bool) -> std::path::PathBuf {
    //

    // Create a Rust-based cross-platform executable wrapper.
    // Instead of using a shell script, we compile a small rust program.
    // However, that might be too slow for a test.
    // Another cross-platform way is to mock it by just using `echo` and `false`/`exit 1`, or
    // we can write a small cargo project and compile it, but that's overkill.
    // Is `sh` available on Windows via MSYS2? Yes usually in CI, but to be truly cross-platform:

    // Instead, we can use a simpler approach: just point the op_bin to a script if unix, or bat file if windows.
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

// Group both scenarios into one test function so they run sequentially
// and don't race on the environment variable.
#[test]
fn op_cli_reader_integration_tests() {
    let temp_dir = TempDir::new().unwrap();

    // 1. Success case
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

    // 2. Failure case
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
