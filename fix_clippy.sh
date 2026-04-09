sed -i 's/-> Result<(), CliError> {/-> () {/g' crates/logos-cli/src/commands/analytics.rs
sed -i 's/Ok(())//g' crates/logos-cli/src/commands/analytics.rs
sed -i 's/-> Result<(), CliError> {/-> () {/g' crates/logos-cli/src/commands/budget.rs
sed -i 's/Ok(())//g' crates/logos-cli/src/commands/budget.rs
sed -i 's/-> Result<(), CliError> {/-> () {/g' crates/logos-cli/src/commands/help.rs
sed -i 's/Ok(())//g' crates/logos-cli/src/commands/help.rs

sed -i 's/pub fn read_event_char/pub(crate) fn read_event_char/g' crates/logos-tui/src/terminal.rs
