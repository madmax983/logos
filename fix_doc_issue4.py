with open('crates/logos-core/src/experimental/mod.rs', 'r') as f:
    content = f.read()

new_content = "//! Experimental features, simulators, and analytics.\n//!\n//! These modules are under active development and are currently behind the `nova` feature flag.\n" + content

with open('crates/logos-core/src/experimental/mod.rs', 'w') as f:
    f.write(new_content)


with open('crates/logos-core/src/format/mod.rs', 'r') as f:
    content = f.read()

new_content = "//! Formatting utilities for displaying currency and timestamps.\n" + content

with open('crates/logos-core/src/format/mod.rs', 'w') as f:
    f.write(new_content)


with open('crates/logos-tui/src/lib.rs', 'r') as f:
    content = f.read()

new_content = "//! Terminal User Interface for Logos\n//!\n//! This crate provides the interactive terminal frontend for exploring ledgers, budgets, and analytical models.\n" + content

with open('crates/logos-tui/src/lib.rs', 'w') as f:
    f.write(new_content)
