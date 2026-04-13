# 🔭 Vantage: Spec for Getting Started Reliability

👤 **User Story:**
"As a new user evaluating Logos, I want to be able to copy-paste the entire block of 'Getting Started' commands from the README into my terminal and have them succeed automatically, so that I can immediately experience the value of the software without encountering technical startup friction."

🤔 **So What?**
The first impression of our software is critical. If a user encounters a giant red connection error on their very first interaction because they copy-pasted the provided tutorial, they will likely assume the software is broken and abandon it. By making the "Getting Started" flow completely robust and copy-paste friendly, we reduce drop-off during the critical evaluation phase. Complexity is a cost; onboarding friction is a lost user.

🎯 **Metric Definition:**
Success = 100% of fresh installations can successfully execute the provided README commands in a single, bulk copy-paste operation without encountering a "Connection refused" or equivalent database startup error.

🔎 **Gap Analysis:**
Currently, our documentation provides a series of commands for local setup (`docker compose up -d db` followed by `cargo run -p logos-cli -- db migrate`). However, Postgres takes a few seconds to accept TCP/IP connections after the container is created. Because the CLI immediately attempts to connect when the commands are run back-to-back, it results in a connection failure. We need to bridge the gap between container initialization and application execution.

✅ **Acceptance Criteria:**
- The commands listed in the README's "Example Commands" section must be copy-pasteable as a single block.
- When pasted as a block, the commands must execute sequentially and successfully without manual delays by the user.
- The `db migrate` command must successfully connect to the database on the first try.
- The solution must be self-contained within the documentation instructions, without requiring the user to install additional health-checking tools.

🚫 **Out of Scope:**
- Adding automatic retry loops into the CLI's core database connection logic just to solve a local onboarding timing issue.
- Changing the underlying database infrastructure from Postgres to an embedded store like SQLite.
