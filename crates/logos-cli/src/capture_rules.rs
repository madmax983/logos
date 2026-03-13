use logos_store_aletheia::model::{StoredCaptureDraft, StoredCaptureStatus, StoredTransaction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureClassification {
    status: StoredCaptureStatus,
    suggested_debit_account: Option<String>,
    suggested_credit_account: Option<String>,
}

impl CaptureClassification {
    #[must_use]
    pub fn new(
        status: StoredCaptureStatus,
        suggested_debit_account: Option<String>,
        suggested_credit_account: Option<String>,
    ) -> Self {
        Self {
            status,
            suggested_debit_account,
            suggested_credit_account,
        }
    }

    #[must_use]
    pub const fn status(&self) -> StoredCaptureStatus {
        self.status
    }

    #[must_use]
    pub fn suggested_debit_account(&self) -> Option<&str> {
        self.suggested_debit_account.as_deref()
    }

    #[must_use]
    pub fn suggested_credit_account(&self) -> Option<&str> {
        self.suggested_credit_account.as_deref()
    }
}

pub fn classify_capture_draft<'a, I>(
    draft: &StoredCaptureDraft,
    history: I,
) -> CaptureClassification
where
    I: IntoIterator<Item = &'a StoredTransaction>,
{
    let (mut debit_account, mut credit_account) = explicit_accounts_for_kind(draft);

    if debit_account.is_none() || credit_account.is_none() {
        if let Some((history_debit, history_credit)) = infer_accounts_from_history(draft, history) {
            if debit_account.is_none() {
                debit_account = Some(history_debit);
            }
            if credit_account.is_none() {
                credit_account = Some(history_credit);
            }
        }
    }

    let status = match (debit_account.is_some(), credit_account.is_some()) {
        (true, true) => StoredCaptureStatus::Ready,
        (true, false) | (false, true) => StoredCaptureStatus::Suggested,
        (false, false) => StoredCaptureStatus::NeedsReview,
    };

    CaptureClassification::new(status, debit_account, credit_account)
}

fn explicit_accounts_for_kind(draft: &StoredCaptureDraft) -> (Option<String>, Option<String>) {
    match draft.kind() {
        "expense" => (
            draft.category_hint().map(str::to_owned),
            draft.from_account_hint().map(str::to_owned),
        ),
        "income" => (
            draft.to_account_hint().map(str::to_owned),
            draft.category_hint().map(str::to_owned),
        ),
        "transfer" | "cash" => (
            draft.to_account_hint().map(str::to_owned),
            draft.from_account_hint().map(str::to_owned),
        ),
        _ => (None, None),
    }
}

fn infer_accounts_from_history<'a, I>(
    draft: &StoredCaptureDraft,
    history: I,
) -> Option<(String, String)>
where
    I: IntoIterator<Item = &'a StoredTransaction>,
{
    let normalized_target = normalize_merchant_memo(draft.merchant_memo());
    let mut matches = history
        .into_iter()
        .filter(|transaction| {
            normalize_merchant_memo(transaction.transaction().description()) == normalized_target
        })
        .collect::<Vec<_>>();
    matches.sort_by(|left, right| left.id().as_str().cmp(right.id().as_str()));
    let matched = matches.pop()?;

    let debit_account = matched
        .transaction()
        .postings()
        .iter()
        .find(|posting| posting.amount() > 0)?
        .account()
        .as_str()
        .to_owned();
    let credit_account = matched
        .transaction()
        .postings()
        .iter()
        .find(|posting| posting.amount() < 0)?
        .account()
        .as_str()
        .to_owned();

    Some((debit_account, credit_account))
}

fn normalize_merchant_memo(raw: &str) -> String {
    raw.split_whitespace()
        .map(|segment| segment.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(" ")
}
