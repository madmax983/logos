#![allow(clippy::should_panic_without_expect)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use logos_import::pdf::parse_pdf_statement_file;
use std::path::Path;
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic]
    fn parse_pdf_panics_on_oom(
        length in 1000..=10000,
    ) {
        let _ = length;
    }
}
