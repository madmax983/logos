use crate::args::{CliError, HelpTopic};

const GENERAL_HELP_TEXT: &str = "\
Usage: ledger <command> [options]

Commands:
  help [command]                    Show general or command help
  aletheia <subcommand>             Manage local AletheiaDB instance
  txn add ...                       Add a transaction
  txn correct ...                   Append correction metadata for an existing transaction
  analytics snapshot ...            Manage immutable analytics artifacts
  import pdf ...                    Import statement rows from a PDF
  import csv ...                    Import statement rows from a CSV file
  fetch list-runs                   List statement fetch runs
  reconcile month                   Reconcile month against statement balances
  month autopilot                   Run import/reconcile/report/close workflow
  close month                       Freeze a month scope with evidence links
  budget set                        Set a budget value
  report month                      Show the current month report
";

const TXN_HELP_TEXT: &str = "\
Usage: ledger txn <subcommand> [options]

Subcommands:
  add --description <text> --debit-account <name> --credit-account <name> --amount-cents <i64>
  correct --supersedes-id <txn-id> --reason <text>

Environment:
  LOGOS_DB_PATH                    Override embedded ledger store path
";

const BUDGET_HELP_TEXT: &str = "\
Usage: ledger budget <subcommand> [options]

Subcommands:
  set --month <YYYY-MM> --budget-cents <i64> --expense-account-prefix <prefix>
                                     Set budget target for month/scope (all optional)
  rsu-plan --quarterly-units <u32> --bear-price-cents <i64> --base-price-cents <i64> --bull-price-cents <i64>
           [--month <YYYY-MM>] [--days-to-vest <u16>] [--fixed-commitments-cents <i64>]
           [--reserve-sweep-pct <u8>] [--investing-sweep-pct <u8>]
                                     Build bear/base/bull RSU budget scenarios with conservative baseline
  monte-carlo --initial-cents <i64> --monthly-contribution-cents <i64> --annual-mean-return <f64> --annual-volatility <f64>
              --months <u16> --paths <u32> [--seed <u64>]
                                     Simulate future net worth outcomes using randomized market returns
";

const REPORT_HELP_TEXT: &str = "\
Usage: ledger report <subcommand> [options]

Subcommands:
  month --month <YYYY-MM> --checking-account <name>
                                     Show month-windowed report (both optional)
";

const IMPORT_HELP_TEXT: &str = "\
Usage: ledger import <subcommand> [options]

Subcommands:
  pdf --file <path> --account <name> [--dry-run] [--ocr]
                                       Import statement rows from PDF text, optionally OCR scanned pages
  csv --file <path> [--source-id <id>] [--timestamp-idx <usize>] [--amount-idx <usize>] [--memo-idx <usize>]
      [--account-idx <usize>] [--category-idx <usize>] [--skip-header] [--dry-run]
                                        Import statement rows from CSV and persist imported statement evidence
";

const FETCH_HELP_TEXT: &str = "\
Usage: ledger fetch <subcommand> [options]

Subcommands:
  list-runs [--month <YYYY-MM>] [--checking-account <name>]
                                       List persisted statement fetch runs with optional filters; use for needs_attention triage
  show-run --run-id <id>
                                       Show one persisted statement fetch run by id, including artifact path and error summary

Environment:
  LOGOS_FETCH_CONFIG_PATH            Override statement source config path; default is sibling statement-sources.toml next to the ledger store
  LOGOS_FETCH_OP_BIN                 Override 1Password CLI executable path used for secret resolution
";

const RECONCILE_HELP_TEXT: &str = "\
Usage: ledger reconcile <subcommand> [options]

Subcommands:
  month --opening-balance-cents <i64> --closing-balance-cents <i64> [--month <YYYY-MM>] [--checking-account <name>]
                                     Reconcile month net flow against statement closing balance
  list [--month <YYYY-MM>] [--checking-account <name>]
                                     List reconciliation runs with optional filters
  show --run-id <id>
                                      Show one reconciliation run by id
";

const MONTH_HELP_TEXT: &str = "\
Usage: ledger month <subcommand> [options]

Subcommands:
  autopilot [--opening-balance-cents <i64> --closing-balance-cents <i64>] [--month <YYYY-MM>] [--checking-account <name>]
            [--statement-pdf <path>] [--ocr] [--allow-variance] [--analytics-artifact-id <id>] --confirm-close
                                       Import(optional) + reconcile + report + close; balances may be omitted when fetched statement metadata is configured

Default fetch config:
  <ledger-store-parent>/statement-sources.toml

Environment:
  LOGOS_FETCH_CONFIG_PATH            Override statement source config path for config-driven month autopilot fetch
";

const CLOSE_HELP_TEXT: &str = "\
Usage: ledger close <subcommand> [options]

Subcommands:
  month --run-id <id> [--month <YYYY-MM>] [--checking-account <name>] [--analytics-artifact-id <id>]
                                     Close one month scope using a reconciliation run and optional analytics artifact
";

const ANALYTICS_HELP_TEXT: &str = "\
Usage: ledger analytics <subcommand> [options]

Subcommands:
  snapshot create [--as-of-valid-us <i64>] [--as-of-tx-us <i64>] [--schema-version <i64>] [--supersedes <artifact-id>]
                                     Create immutable parquet analytics snapshot + Aletheia manifest
  snapshot list                      List known analytics manifests
  snapshot show --artifact-id <id>   Show one manifest
  sankey                             Generate Mermaid Sankey diagram from current transactions
  fire-sim --monthly-expenses-cents <i64> --liquid-assets-cents <i64> --monthly-savings-cents <i64>
                                     Simulate time to Financial Independence

Environment:
  LOGOS_ARTIFACTS_PATH               Override artifact root directory
";

const ALETHEIA_HELP_TEXT: &str = "\
Usage: ledger aletheia <subcommand>

Subcommands:
  start                             Run local aletheia-server via cargo
  status                            Check /status health endpoint

Environment:
  ALETHEIADB_MANIFEST_PATH          Override AletheiaDB Cargo.toml location
  GALLIFREYDB_HOST                  Host used for status checks
  GALLIFREYDB_PORT                  Port used for status checks/server
";

/// Handles help output for general and command-specific help.
///
/// # Errors
///
/// This handler never errors.
pub fn show(topic: HelpTopic) -> Result<(), CliError> {
    let text = help_text(topic);
    println!("{text}");
    Ok(())
}

fn help_text(topic: HelpTopic) -> &'static str {
    match topic {
        HelpTopic::General => GENERAL_HELP_TEXT,
        HelpTopic::Txn => TXN_HELP_TEXT,
        HelpTopic::Analytics => ANALYTICS_HELP_TEXT,
        HelpTopic::Budget => BUDGET_HELP_TEXT,
        HelpTopic::Report => REPORT_HELP_TEXT,
        HelpTopic::Aletheia => ALETHEIA_HELP_TEXT,
        HelpTopic::Import => IMPORT_HELP_TEXT,
        HelpTopic::Fetch => FETCH_HELP_TEXT,
        HelpTopic::Reconcile => RECONCILE_HELP_TEXT,
        HelpTopic::Month => MONTH_HELP_TEXT,
        HelpTopic::Close => CLOSE_HELP_TEXT,
    }
}

#[cfg(test)]
mod tests {
    use super::help_text;
    use crate::args::HelpTopic;

    #[test]
    fn month_help_mentions_default_statement_source_discovery() {
        let text = help_text(HelpTopic::Month);

        assert!(text.contains("<ledger-store-parent>/statement-sources.toml"));
        assert!(text.contains("LOGOS_FETCH_CONFIG_PATH"));
    }

    #[test]
    fn fetch_help_mentions_needs_attention_triage() {
        let text = help_text(HelpTopic::Fetch);

        assert!(text.contains("needs_attention triage"));
        assert!(text.contains("error summary"));
    }

    #[test]
    fn fetch_help_mentions_op_bin_override() {
        let text = help_text(HelpTopic::Fetch);

        assert!(text.contains("LOGOS_FETCH_OP_BIN"));
    }
}
