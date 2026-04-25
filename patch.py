import sys

with open('crates/logos-store-pg/src/store.rs', 'r') as f:
    content = f.read()

search = r"""        let mut postings_by_transaction: HashMap<String, Vec<PostingRow>> =
            HashMap::with_capacity(transaction_ids.len());

        for posting_row in posting_rows {
            // ⚡ Bolt: Using `get_mut` followed by an `insert` fallback avoids an unconditional `.clone()`
            // on the `String` transaction ID for every single posting row.
            // This reduces heap allocations by roughly 50-75% depending on average postings per transaction.
            if let Some(postings) = postings_by_transaction.get_mut(&posting_row.transaction_id) {
                postings.push(posting_row);
            } else {
                postings_by_transaction
                    .insert(posting_row.transaction_id.clone(), vec![posting_row]);
            }
        }

        rows.into_iter()
            .map(|row| {
                let posting_rows = postings_by_transaction.remove(&row.id).ok_or_else(|| {
                    load_failure(format!("transaction '{}' has no postings", row.id))
                })?;
                Self::stored_transaction_from_rows(&row, posting_rows)
            })
            .collect()"""

replace = r"""        let mut postings_by_transaction: HashMap<String, Vec<PostingRow>> =
            HashMap::with_capacity(transaction_ids.len());

        for mut posting_row in posting_rows {
            // ⚡ Bolt: Using `std::mem::take` allows us to extract the `transaction_id` string from the
            // `posting_row` to use as the HashMap key for the first entry without cloning it,
            // significantly reducing heap allocations.
            if let Some(postings) = postings_by_transaction.get_mut(&posting_row.transaction_id) {
                postings.push(posting_row);
            } else {
                let id = std::mem::take(&mut posting_row.transaction_id);
                postings_by_transaction.insert(id, vec![posting_row]);
            }
        }

        rows.into_iter()
            .map(|row| {
                let posting_rows = postings_by_transaction.remove(&row.id).ok_or_else(|| {
                    load_failure(format!("transaction '{}' has no postings", row.id))
                })?;
                Self::stored_transaction_from_rows(&row, posting_rows)
            })
            .collect()"""

new_content = content.replace(search, replace)
if new_content == content:
    print("Failed to replace content", file=sys.stderr)
    sys.exit(1)

with open('crates/logos-store-pg/src/store.rs', 'w') as f:
    f.write(new_content)
