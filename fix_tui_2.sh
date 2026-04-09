sed -i 's/pub mod app;/pub(crate) mod app;/g' crates/logos-tui/src/lib.rs
sed -i 's/pub mod ui;/pub(crate) mod ui;/g' crates/logos-tui/src/lib.rs
sed -i 's/pub mod terminal;/pub(crate) mod terminal;/g' crates/logos-tui/src/lib.rs
