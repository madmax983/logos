import re

with open("crates/logos-core/src/experimental/anomaly_detector.rs", "r") as f:
    content = f.read()

tests = """
    #[test]
    fn test_anomaly_detector_extreme_outliers() {
        let detector = AnomalyDetector::new(3.0);
        let mut transactions = Vec::new();
        let amounts = vec![100, 200, 300, 400, 500];
        for amount in amounts {
            let tx = TransactionBuilder::new("Groceries")
                .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), amount).unwrap())
                .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), amount).unwrap())
                .build()
                .unwrap();
            transactions.push(tx);
        }

        // With 6 elements:
        // Lower half: 100, 200, 300 => Median = 200
        // Upper half: 400, 500, X => Median = 500
        // IQR = 500 - 200 = 300
        // Bound = 500 + 3.0 * 300 = 1400

        // Exact bound should not be outlier
        let tx1 = TransactionBuilder::new("NotAnomaly")
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 1400).unwrap())
            .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 1400).unwrap())
            .build()
            .unwrap();

        let mut txs_1400 = transactions.clone();
        txs_1400.push(tx1);
        let anomalies_1400 = detector.detect(&txs_1400);
        assert_eq!(anomalies_1400.len(), 0);

        // Above bound should be outlier
        let tx2 = TransactionBuilder::new("Anomaly")
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 1401).unwrap())
            .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 1401).unwrap())
            .build()
            .unwrap();
        let mut txs_1401 = transactions.clone();
        txs_1401.push(tx2);

        let anomalies_1401 = detector.detect(&txs_1401);
        assert_eq!(anomalies_1401.len(), 1);
        assert_eq!(anomalies_1401[0].amount_cents, 1401);
    }
}
"""

content = re.sub(r'#\[test\]\s*fn test_anomaly_detector_extreme_outliers\(\).*?(?=^})', tests.strip()[:-1], content, flags=re.MULTILINE | re.DOTALL)

with open("crates/logos-core/src/experimental/anomaly_detector.rs", "w") as f:
    f.write(content)
