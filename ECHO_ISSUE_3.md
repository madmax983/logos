# 🗣️ Echo: `budget monte-carlo` requires percentages as decimals but doesn't say so

**🤦 The Confusion:**
I wanted to simulate my net worth over 10 years, so I ran the `budget monte-carlo` command. I want a 7% annual return and 15% volatility. So I passed `--annual-mean-return 7 --annual-volatility 15`.
The CLI told me my P5 (pessimistic) outcome is `$-77,188,650,876,832,448.00` and my P95 (optimistic) outcome is `$59,457,689,153,718,736.00`. It said my median outcome is `$999.99` even though I started with $1000 and contribute $1000 every month!

**🕵️ The Reality:**
The `budget monte-carlo` command accepts `--annual-mean-return` and `--annual-volatility` as `f64`, but it expects them in decimal format (e.g. `0.07` for 7%). Passing `7` means a 700% annual return and passing `15` means a 1500% annual volatility! The help menu simply says `--annual-mean-return <f64>` and gives no examples. This causes massive integer overflow and complete garbage results if a user passes whole percentages.

**💡 The Fix:**
Either update the CLI to divide the inputs by 100 if they are whole percentages, or at least update the help text/documentation to explicitly state `(e.g., 0.07 for 7%)`.
