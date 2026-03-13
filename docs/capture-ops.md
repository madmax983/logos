# Vault Capture Ops

## Current State

`logos-cli` now supports a draft-first capture inbox flow for phone entry through a synced Obsidian vault:

- `ledger capture ingest --vault-path <path>`
- `ledger capture list [--status ...]`
- `ledger capture show --capture-id <id>`
- `ledger capture promote --capture-id <id> [--debit-account ...] [--credit-account ...]`
- `ledger capture reject --capture-id <id> --reason <text>`

The vault is the transport queue, not the source of ledger truth. Once ingested, the durable record lives in the embedded store configured by `LOGOS_DB_PATH` or the default profile path.
The `vault inbox` flow is built for quick capture from `obsidian mobile` and slower review on the laptop.

## Recommended Vault Layout

Use one note per draft under a dedicated inbox tree:

```text
G:\My Drive\claude\
  finance\
    inbox\
      2026\
        03\
          2026-03-11T18-42-05Z-cap-1.md
    processed\
    rejected\
```

Rules:

- keep one capture per file
- never append multiple captures to one shared daily note
- keep file names sortable and human-readable
- let Google Drive / Obsidian sync the files; Logos only reads Markdown from the inbox tree

Default inbox discovery is `finance/inbox` relative to the vault path you pass to `capture ingest`.

## Note Schema

Each note must start with YAML frontmatter. Required fields:

- `capture_id`
- `captured_at`
- `kind`
- `amount_cents`
- `currency`
- `merchant_memo`

Optional fields:

- `from_account_hint`
- `to_account_hint`
- `category_hint`
- `status`

Current parser rules:

- `captured_at` must be RFC3339
- `kind` must be one of `expense`, `income`, `transfer`, or `cash`
- `currency` defaults to `USD` when omitted
- `status` defaults to `inbox` when omitted
- the body below frontmatter is stored as operator context only

Use the template in [docs/templates/obsidian-capture-note.md](templates/obsidian-capture-note.md) as the canonical shape.

## Example Workflow

Phone capture:

1. Create a new note in Obsidian mobile under `finance/inbox/YYYY/MM/`.
2. Fill in the frontmatter as quickly as possible.
3. Let Obsidian / Google Drive sync.

Laptop ingest and review:

```powershell
ledger capture ingest --vault-path "G:\My Drive\claude"
ledger capture list --status inbox
ledger capture show --capture-id cap-1
```

Promotion when the draft is ready:

```powershell
ledger capture promote --capture-id cap-1
```

Promotion with explicit overrides when rules are insufficient:

```powershell
ledger capture promote --capture-id cap-1 --debit-account expenses:food:dining --credit-account liabilities:amex:gold
```

Rejecting a bad draft:

```powershell
ledger capture reject --capture-id cap-1 --reason "duplicate lunch"
```

## Suggestion and Status Model

The capture workflow uses these statuses:

- `inbox`: newly ingested draft
- `ready`: both sides inferred cleanly from hints or exact history
- `suggested`: one or both sides inferred but not trustworthy enough to auto-post
- `needs_review`: insufficient hints/history
- `promoted`: posted into the real ledger and linked to `txn_id`
- `rejected`: operator intentionally discarded the draft
- `conflict`: a changed payload reappeared after a terminal state

Current suggestion rules are intentionally narrow:

- explicit note hints win first
- exact case-insensitive merchant match against prior transaction descriptions
- no fuzzy merchant matching yet

## Environment and Paths

Relevant runtime knobs:

- `LOGOS_DB_PATH`: optional override for the embedded ledger store

There is intentionally no hidden vault-path environment variable in v1. The vault location is passed explicitly to `ledger capture ingest` so scheduled jobs and local shells cannot disagree silently.

## Common Failure Modes

Malformed frontmatter:

- symptom: `capture ingest` reports `malformed > 0`
- cause: missing frontmatter fence, bad YAML, invalid `kind`, or invalid RFC3339 timestamp
- fix: open the note, repair the frontmatter, rerun ingest

Sync delay:

- symptom: the phone note does not appear on the laptop yet
- cause: Obsidian mobile or Google Drive has not finished syncing
- fix: wait for sync, confirm the file exists under the expected inbox folder, rerun ingest

Changed payload after promotion or rejection:

- symptom: `capture ingest` reports `conflict > 0`
- cause: a note with the same `capture_id` changed after the draft reached a terminal state
- fix: inspect the draft with `capture show`, compare it to the linked transaction or rejection reason, then decide whether to create a new note with a fresh `capture_id`

Missing account resolution on promote:

- symptom: `capture promote` fails asking for a missing debit or credit account
- cause: hints and history were not enough
- fix: rerun with explicit `--debit-account` and `--credit-account`, or update the source note with better hints before ingesting again
