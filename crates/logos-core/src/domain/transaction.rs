use crate::error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posting {
    account: String,
    amount: i64,
}

impl Posting {
    #[must_use]
    pub fn debit(account: &str, amount: i64) -> Self {
        Self {
            account: account.to_owned(),
            amount,
        }
    }

    /// Creates a credit posting, negating the provided amount.
    ///
    /// # Errors
    ///
    /// Returns an error if negating `amount` causes an arithmetic overflow.
    pub fn credit(account: &str, amount: i64) -> Result<Self, DomainError> {
        let amount = amount.checked_neg().ok_or(DomainError::AmountOverflow)?;
        Ok(Self {
            account: account.to_owned(),
            amount,
        })
    }

    #[must_use]
    pub fn account(&self) -> &str {
        &self.account
    }

    #[must_use]
    pub const fn amount(&self) -> i64 {
        self.amount
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    description: String,
    postings: Vec<Posting>,
}

impl Transaction {
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    #[must_use]
    pub fn postings(&self) -> &[Posting] {
        &self.postings
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TransactionBuilder {
    description: String,
    postings: Vec<Posting>,
}

impl TransactionBuilder {
    #[must_use]
    pub fn new(description: &str) -> Self {
        Self {
            description: description.to_owned(),
            postings: Vec::new(),
        }
    }

    #[must_use]
    pub fn posting(mut self, posting: Posting) -> Self {
        self.postings.push(posting);
        self
    }

    /// Builds a transaction after enforcing write-time balancing.
    ///
    /// # Errors
    ///
    /// Returns an error when the description is empty or postings do not sum to zero.
    pub fn build(self) -> Result<Transaction, DomainError> {
        if self.description.trim().is_empty() {
            return Err(DomainError::EmptyTransactionDescription);
        }

        let mut total = 0_i64;
        for posting in &self.postings {
            total = total
                .checked_add(posting.amount())
                .ok_or(DomainError::AmountOverflow)?;
        }

        if total != 0 {
            return Err(DomainError::UnbalancedTransaction { total });
        }

        Ok(Transaction {
            description: self.description,
            postings: self.postings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_error_when_posting_credit_negates_min_value() {
        let result = Posting::credit("income:salary", i64::MIN);
        assert_eq!(result, Err(DomainError::AmountOverflow));
    }

    #[test]
    fn should_return_error_when_transaction_sum_overflows_positive() {
        let builder = TransactionBuilder::new("overflow")
            .posting(Posting::debit("assets:checking", i64::MAX))
            .posting(Posting::debit("assets:checking", 2))
            .posting(Posting::credit("income:salary", i64::MAX).expect("credit"))
            .posting(Posting::credit("income:salary", 2).expect("credit"));

        let result = builder.build();
        assert_eq!(result, Err(DomainError::AmountOverflow));
    }

    #[test]
    fn should_return_error_when_transaction_sum_overflows_negative() {
        let builder = TransactionBuilder::new("underflow")
            .posting(Posting::credit("income:salary", i64::MAX).expect("credit"))
            .posting(Posting::credit("income:salary", 2).expect("credit"))
            .posting(Posting::debit("assets:checking", i64::MAX))
            .posting(Posting::debit("assets:checking", 2));

        let result = builder.build();
        assert_eq!(result, Err(DomainError::AmountOverflow));
    }
}
