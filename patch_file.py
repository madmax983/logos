import re

with open("crates/logos-store-aletheia/src/lib.rs", "r") as f:
    content = f.read()

replacement = """    let mut config = AletheiaDBConfig::builder().wal(wal_config).build();
    config.persistence.data_dir = root_path.join("index-data");

    // Temporarily redirect stderr to /dev/null to suppress AletheiaDB's hardcoded eprintln! calls
    let is_debug = std::env::var("RUST_LOG").is_ok();

    #[cfg(unix)]
    let mut stderr_fd_backup = None;

    #[cfg(unix)]
    if !is_debug {
        use std::os::unix::io::AsRawFd;
        if let Ok(dev_null) = std::fs::File::create("/dev/null") {
            #[allow(unsafe_code)]
            unsafe {
                let old_stderr = libc::dup(libc::STDERR_FILENO);
                if old_stderr != -1 {
                    stderr_fd_backup = Some(old_stderr);
                    libc::dup2(dev_null.as_raw_fd(), libc::STDERR_FILENO);
                }
            }
        }
    }

    let db_result = AletheiaDB::with_unified_config(config).map_err(|err| StoreError::LoadFailed {
        message: format!(
            "unable to initialize embedded AletheiaDB at '{}': {}",
            root_path.display(), err
        ),
    });

    #[cfg(unix)]
    if let Some(old_stderr) = stderr_fd_backup {
        #[allow(unsafe_code)]
        unsafe {
            libc::dup2(old_stderr, libc::STDERR_FILENO);
            libc::close(old_stderr);
        }
    }

    db_result
}"""

original = """    let mut config = AletheiaDBConfig::builder().wal(wal_config).build();
    config.persistence.data_dir = root_path.join("index-data");

    AletheiaDB::with_unified_config(config).map_err(|err| StoreError::LoadFailed {
        message: format!(
            "unable to initialize embedded AletheiaDB at '{}': {err}",
            root_path.display()
        ),
    })
}"""

content = content.replace(original, replacement)

with open("crates/logos-store-aletheia/src/lib.rs", "w") as f:
    f.write(content)
