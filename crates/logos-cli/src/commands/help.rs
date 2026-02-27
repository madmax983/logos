use crate::args::{CliError, HelpTopic};

const GENERAL_HELP_TEXT: &str = "\
Usage: ledger <command> [options]

Commands:
  help [command]                    Show general or command help
  aletheia <subcommand>             Manage local AletheiaDB instance
  txn add ...                       Add a transaction
  analytics snapshot ...            Manage immutable analytics artifacts
  import pdf ...                    Import statement rows from a PDF
  reconcile month                   Reconcile month against statement balances
  close month                       Freeze a month scope with evidence links
  budget set                        Set a budget value
  report month                      Show the current month report
";

const TXN_HELP_TEXT: &str = "\
Usage: ledger txn <subcommand> [options]

Subcommands:
  add --description <text> --debit-account <name> --credit-account <name> --amount-cents <i64>

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

const CLOSE_HELP_TEXT: &str = "\
Usage: ledger close <subcommand> [options]

Subcommands:
  month --run-id <id> [--month <YYYY-MM>] [--checking-account <name>] [--analytics-artifact-id <id>]
                                     Close one month scope using a reconciliation run and optional analytics artifact
";

const ANALYTICS_HELP_TEXT: &str = "\
Usage: ledger analytics snapshot <action> [options]

Actions:
  create [--as-of-valid-us <i64>] [--as-of-tx-us <i64>] [--schema-version <i64>] [--supersedes <artifact-id>]
                                     Create immutable parquet analytics snapshot + Aletheia manifest
  list                               List known analytics manifests
  show --artifact-id <id>            Show one manifest

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
    let text = match topic {
        HelpTopic::General => GENERAL_HELP_TEXT,
        HelpTopic::Txn => TXN_HELP_TEXT,
        HelpTopic::Analytics => ANALYTICS_HELP_TEXT,
        HelpTopic::Budget => BUDGET_HELP_TEXT,
        HelpTopic::Report => REPORT_HELP_TEXT,
        HelpTopic::Aletheia => ALETHEIA_HELP_TEXT,
        HelpTopic::Import => IMPORT_HELP_TEXT,
        HelpTopic::Reconcile => RECONCILE_HELP_TEXT,
        HelpTopic::Close => CLOSE_HELP_TEXT,
    };
    println!("{text}");
    Ok(())
}
