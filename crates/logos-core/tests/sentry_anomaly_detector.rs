use logos_core::domain::account::AccountId;
use logos_core::domain::transaction::{Posting, TransactionBuilder};
use logos_core::experimental::anomaly_detector::AnomalyDetector;

#[test]
fn test_detects_no_anomalies_when_not_enough_data() {
    let detector = AnomalyDetector::new(1.5);
    let mut transactions = Vec::new();

    let amounts = vec![1000, 1200, 1400]; // Only 3 data points, IQR requires at least 4

    for amount in amounts {
        let tx = TransactionBuilder::new("Groceries")
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), amount).unwrap())
            .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), amount).unwrap())
            .build()
            .unwrap();
        transactions.push(tx);
    }

    let anomalies = detector.detect(&transactions);
    assert_eq!(anomalies.len(), 0); // Not enough data points to compute IQR, should silently skip
}
