#![allow(clippy::should_panic_without_expect)]


#[test]
#[should_panic(expected = "attempt to add with overflow")]
fn parse_pdf_statement_file_panics_on_overflow() {
    let _pdf_path = std::env::temp_dir().join(format!("havoc-pdf-{}.pdf", std::process::id()));
    // Let's see if date_index + 1 panics.
    // If date_index == usize::MAX, date_index + 1 will panic. But tokens.len() is bound by memory.
}
