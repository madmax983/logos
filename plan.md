1. **Refactor CLI args**
   - The CLI `args.rs` was a 1300+ line God Module that conflated data modeling (Enums/Structs for commands), argument parsing logic, and execution routing.
   - I will extract the models into `args/model.rs` and the parsers into `args/parser.rs`.
   - I will create a `args/mod.rs` to re-export the extracted elements, ensuring external code does not break.
   - I will verify that `clippy::too_many_lines` and `clippy::cognitive_complexity` are clean.
2. **Update Forge Journal**
   - I will add a new journal entry to `.jules/forge.md` logging the smell and the fix.
3. **Pre-commit Steps**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
4. **Submit**
   - Submit the PR with standard Forge PR conventions.
