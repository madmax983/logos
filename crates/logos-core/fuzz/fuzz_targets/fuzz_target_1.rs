#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = logos_core::Correction::new(logos_core::TransactionId::new("tx").unwrap(), s);
        let _ = logos_core::TransactionId::new(s);
    }
});