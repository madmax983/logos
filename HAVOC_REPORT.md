Title: "👺 Havoc: `parse_simple_csv_row` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Passing `usize::MAX` as indices in `CsvMapping` causes an arithmetic buffer overflow when validating the required column length.

📉 **The Stack Trace:**
```
thread 'parse_simple_csv_row_panics_on_overflow' panicked at crates/logos-import/src/csv.rs:202:18:
attempt to add with overflow
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/4a4ef493e3a1488c6e321570238084b38948f6db/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/4a4ef493e3a1488c6e321570238084b38948f6db/library/core/src/panicking.rs:80:14
   2: core::panicking::panic_const::panic_const_add_overflow
             at /rustc/4a4ef493e3a1488c6e321570238084b38948f6db/library/core/src/panicking.rs:175:17
   3: logos_import::csv::parse_simple_csv_row
             at ./src/csv.rs:202:18
```

🧪 **Reproduction:**
Run `cargo test -p logos-import --test havoc_proptest`

😈 **Comment:**
"You assumed CSV column indices would never exceed hardware limits when computing required sizes. You were wrong."
