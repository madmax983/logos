#![no_main]

use libfuzzer_sys::fuzz_target;
use logos_core::Transaction;

fuzz_target!(|data: &str| {
    // Fuzzing string inputs to trigger Chaos Mode
    let _ = Transaction::new(data);
});
