use logos_import::CsvMapping;
use std::path::Path;

use crate::args::CliError;
use logos_runtime::AppRuntime;

/// Handles `ledger import pdf`.
///
/// # Errors
///
/// Returns an error when runtime initialization or import execution fails.
pub fn pdf(file_path: &str, account: &str, dry_run: bool, ocr: bool) -> Result<(), CliError> {
    let mut runtime = AppRuntime::new().map_err(|err| CliError::CommandRuntimeFailed {
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
    let mut runtime = AppRuntime::new().map_err(|err| CliError::CommandRuntimeFailed {
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
    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header(vec![
        "File",
        "Account",
        "Dry Run",
        "OCR",
        "Imported",
        "Duplicates",
    ]);
    table.add_row(vec![
        file_path.to_string(),
        account.to_string(),
        dry_run.to_string(),
        ocr.to_string(),
        imported_count.to_string(),
        duplicate_count.to_string(),
    ]);

    format!("import.pdf\n{table}")
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
    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header(vec![
        "File",
        "Source ID",
        "Dry Run",
        "Skip Header",
        "Timestamps Idx",
        "Amount Idx",
        "Memo Idx",
        "Account Idx",
        "Category Idx",
        "Imported",
        "Duplicates",
    ]);
    table.add_row(vec![
        file_path.to_string(),
        source_id.to_string(),
        dry_run.to_string(),
        skip_header.to_string(),
        timestamp_idx.to_string(),
        amount_idx.to_string(),
        memo_idx.to_string(),
        account_idx.to_string(),
        category_idx.to_string(),
        imported_count.to_string(),
        duplicate_count.to_string(),
    ]);

    format!("import.csv\n{table}")
}

#[cfg(test)]
mod tests {
    use super::{render_csv_output, render_pdf_output};

    #[test]
    fn render_pdf_output_is_deterministic() {
        let output = render_pdf_output("stmt.pdf", "assets:checking", true, false, 12, 3);
        assert!(output.contains("import.pdf"));
        assert!(output.contains("stmt.pdf"));
        assert!(output.contains("assets:checking"));
        assert!(output.contains("true"));
        assert!(output.contains("false"));
        assert!(output.contains("12"));
        assert!(output.contains('3'));
    }

    #[test]
    fn render_csv_output_is_deterministic() {
        let output = render_csv_output(
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
        );
        assert!(output.contains("import.csv"));
        assert!(output.contains("statement.csv"));
        assert!(output.contains("chase.csv"));
        assert!(output.contains("false"));
        assert!(output.contains("true"));
        assert!(output.contains('0'));
        assert!(output.contains('1'));
        assert!(output.contains('2'));
        assert!(output.contains('3'));
        assert!(output.contains('4'));
        assert!(output.contains('9'));
        assert!(output.contains('1'));
    }
}
