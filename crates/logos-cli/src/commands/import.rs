use logos_import::CsvMapping;
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

/// Handles `ledger import csv`.
///
/// # Errors
///
/// Returns an error when runtime initialization or import execution fails.
#[allow(clippy::too_many_arguments)]
pub fn csv(
    file_path: &str,
    source_id: Option<&str>,
    timestamp_idx: usize,
    amount_idx: usize,
    memo_idx: usize,
    account_idx: usize,
    category_idx: usize,
    skip_header: bool,
    dry_run: bool,
) -> Result<(), CliError> {
    let mut runtime = CliRuntime::new().map_err(|err| CliError::CommandRuntimeFailed {
        command: "import.csv".to_owned(),
        message: format!("runtime initialization failed: {err}"),
    })?;
    let source_id = source_id.map_or_else(
        || {
            Path::new(file_path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(file_path)
                .to_owned()
        },
        str::to_owned,
    );
    let mapping = CsvMapping {
        source_id,
        timestamp_idx,
        amount_idx,
        memo_idx,
        account_idx,
        category_idx,
    };
    let summary = runtime
        .import_csv_statement(Path::new(file_path), &mapping, dry_run, skip_header)
        .map_err(|err| CliError::CommandRuntimeFailed {
            command: "import.csv".to_owned(),
            message: err.to_string(),
        })?;

    println!(
        "{}",
        render_csv_output(
            file_path,
            mapping.source_id.as_str(),
            dry_run,
            skip_header,
            mapping.timestamp_idx,
            mapping.amount_idx,
            mapping.memo_idx,
            mapping.account_idx,
            mapping.category_idx,
            summary.imported_count(),
            summary.duplicate_count(),
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

#[allow(clippy::too_many_arguments)]
fn render_csv_output(
    file_path: &str,
    source_id: &str,
    dry_run: bool,
    skip_header: bool,
    timestamp_idx: usize,
    amount_idx: usize,
    memo_idx: usize,
    account_idx: usize,
    category_idx: usize,
    imported_count: usize,
    duplicate_count: usize,
) -> String {
    format!(
        "import.csv file={file_path} source_id={source_id} dry_run={dry_run} skip_header={skip_header} timestamp_idx={timestamp_idx} amount_idx={amount_idx} memo_idx={memo_idx} account_idx={account_idx} category_idx={category_idx} imported={imported_count} duplicates={duplicate_count}"
    )
}

#[cfg(test)]
mod tests {
    use super::{render_csv_output, render_pdf_output};

    #[test]
    fn render_pdf_output_is_deterministic() {
        assert_eq!(
            render_pdf_output("stmt.pdf", "assets:checking", true, false, 12, 3),
            "import.pdf file=stmt.pdf account=assets:checking dry_run=true ocr=false imported=12 duplicates=3"
        );
    }

    #[test]
    fn render_csv_output_is_deterministic() {
        assert_eq!(
            render_csv_output(
                "statement.csv",
                "chase.csv",
                false,
                true,
                0,
                1,
                2,
                3,
                4,
                9,
                1,
            ),
            "import.csv file=statement.csv source_id=chase.csv dry_run=false skip_header=true timestamp_idx=0 amount_idx=1 memo_idx=2 account_idx=3 category_idx=4 imported=9 duplicates=1"
        );
    }
}
