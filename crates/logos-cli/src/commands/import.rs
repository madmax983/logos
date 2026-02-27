use std::path::Path;

use crate::{args::CliError, runtime::CliRuntime};

/// Handles `ledger import pdf`.
///
/// # Errors
///
/// Returns an error when runtime initialization or import execution fails.
pub fn pdf(file_path: &str, account: &str, dry_run: bool, ocr: bool) -> Result<(), CliError> {
    let mut runtime = CliRuntime::new().map_err(|err| CliError::CommandRuntimeFailed {
        command: "import.pdf".to_owned(),
        message: format!("runtime initialization failed: {err}"),
    })?;
    let summary = runtime
        .import_pdf_statement(Path::new(file_path), account, dry_run, ocr)
        .map_err(|err| CliError::CommandRuntimeFailed {
            command: "import.pdf".to_owned(),
            message: err.to_string(),
        })?;

    println!(
        "{}",
        render_pdf_output(
            file_path,
            account,
            dry_run,
            ocr,
            summary.imported_count(),
            summary.duplicate_count()
        )
    );
    Ok(())
}

fn render_pdf_output(
    file_path: &str,
    account: &str,
    dry_run: bool,
    ocr: bool,
    imported_count: usize,
    duplicate_count: usize,
) -> String {
    format!(
        "import.pdf file={file_path} account={account} dry_run={dry_run} ocr={ocr} imported={imported_count} duplicates={duplicate_count}"
    )
}

#[cfg(test)]
mod tests {
    use super::render_pdf_output;

    #[test]
    fn render_pdf_output_is_deterministic() {
        assert_eq!(
            render_pdf_output("stmt.pdf", "assets:checking", true, false, 12, 3),
            "import.pdf file=stmt.pdf account=assets:checking dry_run=true ocr=false imported=12 duplicates=3"
        );
    }
}
