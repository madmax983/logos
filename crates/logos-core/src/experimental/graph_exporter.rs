#![cfg(feature = "nova")]

//! Graph Exporter
//!
//! Exports transactions as a Graphviz DOT format to visualize the flow of money.

use crate::domain::transaction::Transaction;
use std::collections::HashMap;
use std::fmt::Write;

/// Exports a collection of transactions into a Graphviz DOT diagram.
///
/// This provides a visual network representation of cashflow.
#[derive(Debug, Default)]
pub struct DotGraphExporter<'a> {
    transactions: Vec<&'a Transaction>,
}

impl<'a> DotGraphExporter<'a> {
    /// Creates a new, empty `DotGraphExporter`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a transaction to the exporter.
    pub fn add_transaction(&mut self, transaction: &'a Transaction) {
        self.transactions.push(transaction);
    }

    /// Generates a Graphviz DOT string representing the aggregated cashflow.
    #[must_use]
    pub fn export_dot(&self) -> String {
        let mut flows: HashMap<(&str, &str), i64> = HashMap::new();

        for tx in &self.transactions {
            let total_credit: i64 = tx
                .postings()
                .iter()
                .filter(|p| p.amount() < 0)
                .map(|p| p.amount().abs())
                .fold(0_i64, i64::saturating_add);

            if total_credit == 0 {
                continue;
            }

            for credit in tx.postings().iter().filter(|p| p.amount() < 0) {
                let credit_amount = credit.amount().abs();
                #[allow(clippy::cast_precision_loss)]
                let credit_proportion = credit_amount as f64 / total_credit as f64;

                for debit in tx.postings().iter().filter(|p| p.amount() >= 0) {
                    let debit_amount = debit.amount();
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

        let mut output = String::from("digraph Cashflow {\n");
        let _ = writeln!(output, "    node [shape=box];");

        let mut sorted_keys: Vec<_> = flows.keys().collect();
        sorted_keys.sort();

        for key in sorted_keys {
            let amount_cents = flows[key];
            #[allow(clippy::cast_precision_loss)]
            let amount_dollars = amount_cents as f64 / 100.0;
            let _ = writeln!(
                output,
                "    \"{}\" -> \"{}\" [label=\"${:.2}\"];",
                key.0, key.1, amount_dollars
            );
        }

        output.push_str("}\n");
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::AccountId;
    use crate::domain::transaction::{Posting, TransactionBuilder};

    #[test]
    fn test_simple_transfer() {
        let mut exporter = DotGraphExporter::new();

        let tx = TransactionBuilder::new("Salary")
            .posting(Posting::credit(AccountId::new("income:salary").unwrap(), 500_000).unwrap())
            .posting(Posting::debit(AccountId::new("assets:checking").unwrap(), 500_000).unwrap())
            .build()
            .unwrap();

        exporter.add_transaction(&tx);

        let expected = "\
digraph Cashflow {
    node [shape=box];
    \"income:salary\" -> \"assets:checking\" [label=\"$5000.00\"];
}
";
        assert_eq!(exporter.export_dot(), expected);
    }
}
