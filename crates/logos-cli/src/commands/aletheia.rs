use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use crate::args::CliError;

const DEFAULT_ALETHEIA_MANIFEST_PATH: &str = "../AletheiaDB/Cargo.toml";
const DEFAULT_STATUS_HOST: &str = "127.0.0.1";
const DEFAULT_STATUS_PORT: u16 = 8080;
const STATUS_PATH: &str = "/status";
const IO_TIMEOUT_SECONDS: u64 = 2;

/// Handles `ledger aletheia start`.
///
/// # Errors
///
/// Returns an error when the local `AletheiaDB` manifest is missing or launching the
/// server command fails.
pub fn start() -> Result<(), CliError> {
    let manifest_path = resolve_manifest_path();
    if !Path::new(&manifest_path).is_file() {
        return Err(CliError::AletheiaStartFailed {
            message: format!(
                "manifest not found at '{manifest_path}' (override with ALETHEIADB_MANIFEST_PATH)"
            ),
        });
    }

    let status = Command::new("cargo")
        .arg("run")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("--bin")
        .arg("aletheia-server")
        .arg("--features")
        .arg("http-server")
        .status()
        .map_err(|err| CliError::AletheiaStartFailed {
            message: format!("unable to launch cargo: {err}"),
        })?;

    if status.success() {
        return Ok(());
    }

    Err(CliError::AletheiaStartFailed {
        message: format!("server process exited with status {status}"),
    })
}

/// Handles `ledger aletheia status`.
///
/// # Errors
///
/// Returns an error when the status endpoint is unreachable or does not report
/// a healthy payload.
pub fn status() -> Result<(), CliError> {
    let host = resolve_status_host();
    let port = resolve_status_port();
    let endpoint = format!("http://{host}:{port}{STATUS_PATH}");
    let address = format!("{host}:{port}");
    let mut stream =
        TcpStream::connect(&address).map_err(|err| CliError::AletheiaStatusFailed {
            endpoint: endpoint.clone(),
            message: format!("connection failed: {err}"),
        })?;

    stream
        .set_read_timeout(Some(Duration::from_secs(IO_TIMEOUT_SECONDS)))
        .map_err(|err| CliError::AletheiaStatusFailed {
            endpoint: endpoint.clone(),
            message: format!("failed setting read timeout: {err}"),
        })?;
    stream
        .set_write_timeout(Some(Duration::from_secs(IO_TIMEOUT_SECONDS)))
        .map_err(|err| CliError::AletheiaStatusFailed {
            endpoint: endpoint.clone(),
            message: format!("failed setting write timeout: {err}"),
        })?;

    let request =
        format!("GET {STATUS_PATH} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .map_err(|err| CliError::AletheiaStatusFailed {
            endpoint: endpoint.clone(),
            message: format!("request write failed: {err}"),
        })?;
    stream
        .flush()
        .map_err(|err| CliError::AletheiaStatusFailed {
            endpoint: endpoint.clone(),
            message: format!("request flush failed: {err}"),
        })?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|err| CliError::AletheiaStatusFailed {
            endpoint: endpoint.clone(),
            message: format!("response read failed: {err}"),
        })?;

    if response.contains("\"status\":\"healthy\"") || response.contains("\"status\": \"healthy\"") {
        println!("aletheia.status healthy ({endpoint})");
        return Ok(());
    }

    Err(CliError::AletheiaStatusFailed {
        endpoint,
        message: "unexpected status payload (expected JSON status=healthy)".to_owned(),
    })
}

fn resolve_manifest_path() -> String {
    manifest_path_from_env(env::var("ALETHEIADB_MANIFEST_PATH").ok())
}

fn resolve_status_host() -> String {
    status_host_from_env(env::var("GALLIFREYDB_HOST").ok())
}

fn resolve_status_port() -> u16 {
    status_port_from_env(env::var("GALLIFREYDB_PORT").ok())
}

fn manifest_path_from_env(raw: Option<String>) -> String {
    let Some(path) = raw.filter(|value| !value.trim().is_empty()) else {
        return DEFAULT_ALETHEIA_MANIFEST_PATH.to_owned();
    };
    path
}

fn status_host_from_env(raw: Option<String>) -> String {
    match raw {
        Some(value) if !value.trim().is_empty() => {
            if value.trim() == "0.0.0.0" {
                DEFAULT_STATUS_HOST.to_owned()
            } else {
                value
            }
        }
        _ => DEFAULT_STATUS_HOST.to_owned(),
    }
}

fn status_port_from_env(raw: Option<String>) -> u16 {
    raw.and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(DEFAULT_STATUS_PORT)
}

#[cfg(test)]
mod tests {
    use super::{manifest_path_from_env, status_host_from_env, status_port_from_env};

    #[test]
    fn manifest_path_uses_default_when_env_missing() {
        let path = manifest_path_from_env(None);
        assert!(path.ends_with("AletheiaDB/Cargo.toml"));
    }

    #[test]
    fn manifest_path_uses_env_override() {
        let path = manifest_path_from_env(Some(r"D:\custom\Cargo.toml".to_owned()));
        assert_eq!(path, r"D:\custom\Cargo.toml");
    }

    #[test]
    fn status_host_normalizes_wildcard_to_localhost() {
        let host = status_host_from_env(Some("0.0.0.0".to_owned()));
        assert_eq!(host, "127.0.0.1");
    }

    #[test]
    fn status_port_falls_back_on_invalid_env_value() {
        let port = status_port_from_env(Some("invalid".to_owned()));
        assert_eq!(port, 8080);
    }

    #[test]
    fn status_port_uses_env_value_when_valid() {
        let port = status_port_from_env(Some("4000".to_owned()));
        assert_eq!(port, 4000);
    }
}
