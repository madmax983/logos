use logos_core::experimental::benford_law::BenfordLawAnalyzer;

#[test]
fn test_benford_law_empty_transactions() {
    let mut analyzer = BenfordLawAnalyzer::new();
    analyzer.add_transactions(&[]);

    let observed = analyzer.observed_distribution();
    assert!(observed.is_empty()); // Should not panic or divide by zero when total_samples is 0
}
