with open('crates/logos-fetch/src/lib.rs', 'r') as f:
    content = f.read()

new_content = content.replace("pub use model::{FetchRunStatus, FetchedStatementArtifact, OutputFormat, StatementSource};", "pub use model::{FetchRunStatus, FetchedStatementArtifact, OutputFormat, StatementSource, is_valid_month_key};")

with open('crates/logos-fetch/src/lib.rs', 'w') as f:
    f.write(new_content)
