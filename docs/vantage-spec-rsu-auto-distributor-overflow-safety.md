# 🔭 Vantage: Spec for RSU Auto Distributor Overflow Safety

👤 **User Story:**
"As an employee with RSUs, I want the system to safely distribute massive vests without crashing, so that a large vesting event does not break the transaction generation process."

🤔 **So What?**
A robust financial planning system must not panic on edge-case inputs. When distributing a large RSU vest across multiple goal envelopes based on percentage policies, intermediate multiplication steps can cause an arithmetic overflow if the gross vest is extraordinarily large. Hard crashes erode user trust.

🎯 **Metric Definition:**
Success = 0 panics during RSU distribution via `distribute_rsu_vest`, even when `gross_vest` approaches `i64::MAX`. The system must complete the transaction build or return a clean error without panicking.

🔎 **Gap Analysis:**
Currently, `RsuAutoDistributor::distribute_rsu_vest` crashes with an `attempt to multiply with overflow` panic when processing extremely large values, as demonstrated by the `distribute_rsu_vest_panics_on_overflow` test in `crates/logos-core/tests/havoc_proptest.rs`. Standard financial processors use safe bounds or saturating arithmetic for intermediate percentage calculations to avoid this.

✅ **Acceptance Criteria:**
- The `distribute_rsu_vest` method must not panic when computing percentage allocations for large vest amounts.
- Intermediate multiplication steps must be protected against overflow.

🚫 **Out of Scope:**
- Upgrading the entire underlying storage engine to arbitrary precision integers.
