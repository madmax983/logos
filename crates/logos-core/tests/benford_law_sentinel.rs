#[cfg(feature = "nova")]
#[cfg(test)]
mod tests {
    use logos_core::experimental::benford_law::BenfordLawAnalyzer;
    use logos_core::domain::transaction::{Posting, TransactionBuilder};
    use logos_core::AccountId;

    #[test]
    fn test_sentinel_benford_extract_loop() {
        let mut analyzer = BenfordLawAnalyzer::new();
        let tx = TransactionBuilder::new("Test")
            .posting(Posting::credit(AccountId::new("income:salary").unwrap(), 123).unwrap())
            .posting(Posting::debit(AccountId::new("assets:checking").unwrap(), 123).unwrap())
            .build()
            .unwrap();

        analyzer.add_transactions(&[tx]);
        let dist = analyzer.observed_distribution();

        // This ensures the value isn't stuck in an infinite while loop due to >= mutation
        // Since `123` starts with `1`, there should be two occurrences of `1` across the 2 postings
        let mut expected = 0.0;
        if let Some(pct) = dist.get(&1) {
            expected = *pct;
        }

        // 2 total postings, both start with 1, meaning 1 is 100% (1.0)
        assert_eq!(expected, 1.0);
    }
}
