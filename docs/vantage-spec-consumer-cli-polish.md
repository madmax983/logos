# 🔭 Vantage: Spec for Consumer-Grade CLI Polish

## 👤 **User Story:**
As a new user, I want to start the application out-of-the-box without configuring environment variables and understand the console output, so that I can quickly begin tracking my finances without feeling overwhelmed by database jargon.

## 🤔 **So What?**
What business problem does this solve?
Currently, our onboarding experience is actively hostile to new users. The default Aletheia DB path points to a hardcoded developer directory (`C:\Users\markm\...`), instantly breaking the "Getting Started" flow. Furthermore, standard CLI usage bombards users with complex internals like "Temporal adjacency index restored". By fixing these, we decrease time-to-value for new users, reduce support tickets, and improve product adoption. Complexity is a cost; utility is revenue.

## 🎯 **Metric Definition:**
- **Success:** 100% of fresh installs can run `cargo run -p logos-cli -- aletheia start` successfully without setting `ALETHEIADB_MANIFEST_PATH`.
- **Success:** 0 occurrences of the words "temporal", "adjacency", or "index" in default non-debug CLI stdout during standard commands (`budget set`, `report month`).

## 🔍 **Gap Analysis:**
Looking at the market, successful CLI tools pride themselves on zero-config local onboarding and human-readable output. Currently, we operate more like an academic database prototype. Standard CLI frameworks provide easy ways to filter debug jargon from standard output, which we are not utilizing effectively.

## ✅ **Acceptance Criteria:**
- Must provide a universally valid fallback for the local database server path.
- Must hide technical database initialization jargon from the user's standard output.
- Must only display jargon if a verbose debug mode is explicitly enabled.

## 🚫 **Out of Scope:**
- Complete overhaul of the database architecture.
- Replacing Aletheia DB with another storage engine.
- Graphical User Interface (GUI) onboarding flows.
