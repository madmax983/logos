use logos_import::CsvMapping;
use std::path::Path;

use crate::args::CliError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfImportConfig<'a> {
    pub file_path: &'a str,
    pub account: &'a str,
    pub dry_run: bool,
    pub ocr: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvImportConfig<'a> {
    pub file_path: &'a str,
    pub source_id: Option<&'a str>,
    pub timestamp_idx: usize,
    pub amount_idx: usize,
    pub memo_idx: usize,
    pub account_idx: usize,
    pub category_idx: usize,
    pub skip_header: bool,
    pub dry_run: bool,
}

/// Handles `ledger import pdf`.
///
/// # Errors
///
/// Returns an error when runtime initialization or import execution fails.
pub fn pdf(config: &PdfImportConfig<'_>) -> Result<(), CliError> {
    let mut runtime =
        crate::runtime::init_runtime().map_err(|err| CliError::CommandRuntimeFailed {
            command: "import.pdf".to_owned(),
            message: format!("{err}"),
        })?;
    let summary = runtime
        .import_pdf_statement(
            Path::new(config.file_path),
            config.account,
            config.dry_run,
            config.ocr,
        )
        .map_err(|err| CliError::CommandRuntimeFailed {
            command: "import.pdf".to_owned(),
            message: err.to_string(),
        })?;

    println!(
        "{}",
        render_pdf_output(config, summary.imported_count(), summary.duplicate_count())
    );
    Ok(())
}

/// Handles `ledger import csv`.
///
/// # Errors
///
/// Returns an error when runtime initialization or import execution fails.
pub fn csv(config: &CsvImportConfig<'_>) -> Result<(), CliError> {
    let mut runtime =
        crate::runtime::init_runtime().map_err(|err| CliError::CommandRuntimeFailed {
            command: "import.csv".to_owned(),
            message: format!("{err}"),
        })?;
    let source_id = config.source_id.map_or_else(
        || {
            Path::new(config.file_path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(config.file_path)
                .to_owned()
        },
        str::to_owned,
    );
    let mapping = CsvMapping {
        source_id,
        timestamp_idx: config.timestamp_idx,
        amount_idx: config.amount_idx,
        memo_idx: config.memo_idx,
        account_idx: config.account_idx,
        category_idx: config.category_idx,
    };
    let summary = runtime
        .import_csv_statement(
            Path::new(config.file_path),
            &mapping,
            config.dry_run,
            config.skip_header,
        )
        .map_err(|err| CliError::CommandRuntimeFailed {
            command: "import.csv".to_owned(),
            message: err.to_string(),
        })?;

    println!(
        "{}",
        render_csv_output(
            config,
            mapping.source_id.as_str(),
            summary.imported_count(),
            summary.duplicate_count(),
        )
    );
    Ok(())
}

fn render_pdf_output(
    config: &PdfImportConfig<'_>,
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
        config.file_path.to_string(),
        config.account.to_string(),
        config.dry_run.to_string(),
        config.ocr.to_string(),
        imported_count.to_string(),
        duplicate_count.to_string(),
    ]);

    format!("import.pdf\n{table}")
}

fn render_csv_output(
    config: &CsvImportConfig<'_>,
    source_id: &str,
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
        config.file_path.to_string(),
        source_id.to_string(),
        config.dry_run.to_string(),
        config.skip_header.to_string(),
        config.timestamp_idx.to_string(),
        config.amount_idx.to_string(),
        config.memo_idx.to_string(),
        config.account_idx.to_string(),
        config.category_idx.to_string(),
        imported_count.to_string(),
        duplicate_count.to_string(),
    ]);

    format!("import.csv\n{table}")
}

#[cfg(test)]
mod tests {
    use super::{CsvImportConfig, PdfImportConfig, render_csv_output, render_pdf_output};

    #[test]
    fn render_pdf_output_is_deterministic() {
        let config = PdfImportConfig {
            file_path: "stmt.pdf",
            account: "assets:checking",
            dry_run: true,
            ocr: false,
        };
        let output = render_pdf_output(&config, 12, 3);
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
        let config = CsvImportConfig {
            file_path: "statement.csv",
            source_id: None,
            timestamp_idx: 0,
            amount_idx: 1,
            memo_idx: 2,
            account_idx: 3,
            category_idx: 4,
            skip_header: true,
            dry_run: false,
        };
        let output = render_csv_output(&config, "chase.csv", 9, 1);
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
