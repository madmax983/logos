# Required: point logos at Postgres
export DATABASE_URL="postgres://logos:logos@127.0.0.1:5432/logos"

# Optional local convenience database
docker compose up -d db
sleep 3

# Apply explicit migrations
cargo run -p logos-cli -- db migrate

cargo run -p logos-cli -- txn add --description "paycheck" --debit-account "assets:checking" --credit-account "income:salary" --amount-cents 100000
cargo run -p logos-cli -- budget set --month 2026-03 --budget-cents 300000 --expense-account-prefix "expenses:"
cargo run -p logos-cli -- report month --month 2026-03 --checking-account "assets:checking"
cargo run -p logos-tui
