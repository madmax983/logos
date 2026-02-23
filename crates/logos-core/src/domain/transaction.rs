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

    #[must_use]
    pub fn credit(account: &str, amount: i64) -> Self {
        Self {
            account: account.to_owned(),
            amount: -amount,
        }
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

        let total: i64 = self.postings.iter().map(Posting::amount).sum();
        if total != 0 {
            return Err(DomainError::UnbalancedTransaction { total });
        }

        Ok(Transaction {
            description: self.description,
            postings: self.postings,
        })
    }
}
