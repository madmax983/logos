echo "pub mod emergency_fund_simulator;" >> crates/logos-core/src/experimental/mod.rs
cargo test --workspace --features nova
