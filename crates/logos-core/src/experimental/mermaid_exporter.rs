use std::collections::HashMap;
use std::fmt::Write;

use crate::domain::transaction::{Posting, Transaction};

/// Exports a collection of transactions into a Mermaid Sankey diagram.
///
/// This provides a visual representation of cashflow, showing how money moves
/// from credit accounts (sources) to debit accounts (destinations).
#[derive(Debug, Default)]
pub struct MermaidSankeyExporter {
    transactions: Vec<Transaction>,
}

impl MermaidSankeyExporter {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a transaction to the exporter.
    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    /// Generates a Mermaid Sankey diagram representing the aggregated cashflow.
    ///
    /// The algorithm proportionally distributes credits to debits within each
    /// transaction to determine flow.
    #[must_use]
    pub fn export_sankey(&self) -> String {
        let mut flows: HashMap<(&str, &str), i64> = HashMap::new();

        for tx in &self.transactions {
            let (credits, debits): (Vec<&Posting>, Vec<&Posting>) =
                tx.postings().iter().partition(|p| p.amount() < 0);

            let total_credit: i64 = credits.iter().map(|p| p.amount().abs()).sum();

            if total_credit == 0 {
                continue; // Prevent division by zero, though valid txns shouldn't have 0 total
            }

            for credit in &credits {
                let credit_amount = credit.amount().abs();

                // Determine the proportion of the total credit pool this specific credit represents
                #[allow(clippy::cast_precision_loss)]
                let credit_proportion = credit_amount as f64 / total_credit as f64;

                for debit in &debits {
                    let debit_amount = debit.amount();

                    // The flow from this credit to this debit is the debit's amount multiplied by the credit's proportion
                    #[allow(clippy::cast_precision_loss)]
                    let raw_flow = debit_amount as f64 * credit_proportion;

                    #[allow(clippy::cast_possible_truncation)]
                    let flow_amount = raw_flow.round() as i64;

                    if flow_amount > 0 {
                        let key = (credit.account().as_str(), debit.account().as_str());
                        *flows.entry(key).or_insert(0) += flow_amount;
                    }
                }
            }
        }

        let mut output = String::from("```mermaid\nsankey-beta\n");

        // Sort keys for deterministic output
        let mut sorted_keys: Vec<_> = flows.keys().collect();
        sorted_keys.sort();

        for key in sorted_keys {
            let amount_cents = flows[key];
            #[allow(clippy::cast_precision_loss)]
            let amount_dollars = amount_cents as f64 / 100.0;
            // Mermaid Sankey format: source,target,value
            let _ = writeln!(output, "{},{},{:.2}", key.0, key.1, amount_dollars);
        }

        output.push_str("```\n");
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AccountId;
    use crate::domain::transaction::TransactionBuilder;

    #[test]
    fn test_simple_transfer() {
        let mut exporter = MermaidSankeyExporter::new();

        let tx = TransactionBuilder::new("Salary")
            .posting(Posting::credit(AccountId::new("income:salary").unwrap(), 500_000).unwrap())
            .posting(Posting::debit(
                AccountId::new("assets:checking").unwrap(),
                500_000,
            ))
            .build()
            .unwrap();

        exporter.add_transaction(tx);

        let expected = "\
```mermaid
sankey-beta
income:salary,assets:checking,5000.00
```
";
        assert_eq!(exporter.export_sankey(), expected);
    }

    #[test]
    fn test_split_transaction() {
        let mut exporter = MermaidSankeyExporter::new();

        let tx = TransactionBuilder::new("Split")
            .posting(Posting::credit(AccountId::new("income:salary").unwrap(), 100_000).unwrap())
            .posting(Posting::debit(
                AccountId::new("assets:checking").unwrap(),
                70_000,
            ))
            .posting(Posting::debit(
                AccountId::new("assets:savings").unwrap(),
                30_000,
            ))
            .build()
            .unwrap();

        exporter.add_transaction(tx);

        let expected = "\
```mermaid
sankey-beta
income:salary,assets:checking,700.00
income:salary,assets:savings,300.00
```
";
        assert_eq!(exporter.export_sankey(), expected);
    }

    #[test]
    fn test_multiple_sources() {
        let mut exporter = MermaidSankeyExporter::new();

        let tx = TransactionBuilder::new("Pool")
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 60_000).unwrap())
            .posting(Posting::credit(AccountId::new("assets:savings").unwrap(), 40_000).unwrap())
            .posting(Posting::debit(
                AccountId::new("expenses:rent").unwrap(),
                100_000,
            ))
            .build()
            .unwrap();

        exporter.add_transaction(tx);

        let expected = "\
```mermaid
sankey-beta
assets:checking,expenses:rent,600.00
assets:savings,expenses:rent,400.00
```
";
        assert_eq!(exporter.export_sankey(), expected);
    }

    #[test]
    fn test_complex_proportional_distribution() {
        let mut exporter = MermaidSankeyExporter::new();

        // 60% from checking, 40% from savings
        let tx = TransactionBuilder::new("Complex")
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 6000).unwrap())
            .posting(Posting::credit(AccountId::new("assets:savings").unwrap(), 4000).unwrap())
            // Distributed to:
            .posting(Posting::debit(
                AccountId::new("expenses:food").unwrap(),
                5000,
            )) // 60% of 50 = 30 from checking, 20 from savings
            .posting(Posting::debit(
                AccountId::new("expenses:fun").unwrap(),
                5000,
            )) // 60% of 50 = 30 from checking, 20 from savings
            .build()
            .unwrap();

        exporter.add_transaction(tx);

        let expected = "\
```mermaid
sankey-beta
assets:checking,expenses:food,30.00
assets:checking,expenses:fun,30.00
assets:savings,expenses:food,20.00
assets:savings,expenses:fun,20.00
```
";
        assert_eq!(exporter.export_sankey(), expected);
    }

    #[test]
    fn test_aggregate_multiple_transactions() {
        let mut exporter = MermaidSankeyExporter::new();

        let tx1 = TransactionBuilder::new("T1")
            .posting(Posting::credit(AccountId::new("income").unwrap(), 100).unwrap())
            .posting(Posting::debit(AccountId::new("checking").unwrap(), 100))
            .build()
            .unwrap();

        let tx2 = TransactionBuilder::new("T2")
            .posting(Posting::credit(AccountId::new("income").unwrap(), 200).unwrap())
            .posting(Posting::debit(AccountId::new("checking").unwrap(), 200))
            .build()
            .unwrap();

        exporter.add_transaction(tx1);
        exporter.add_transaction(tx2);

        let expected = "\
```mermaid
sankey-beta
income,checking,3.00
```
";
        assert_eq!(exporter.export_sankey(), expected);
    }
}
