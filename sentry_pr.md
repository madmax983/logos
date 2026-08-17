Title: "🛡️ Sentry: Add Display and From unit tests for RuntimeError"

Description:
🎯 Target: `logos-runtime::error::RuntimeError`
💣 Risk: `RuntimeError`'s `Display` implementation had missed arms and its conversion traits (`From`) lacked unit tests, which could hide serialization/formatting errors in production logs.
🧪 Strategy: Added unit tests for `fmt::Display` and `From` trait implementations to ensure all variants format correctly and conversions succeed.
🔬 Verification: `cargo test --package logos-runtime --lib error`
