# 🔭 Vantage: Spec for Lifestyle Creep Simulator

## The "So What?" (Business Problem)
As users progress in their careers and increase their income, they often proportionally increase their spending (Lifestyle Creep). This hidden tax on wealth generation silently extends the time to FIRE. Users lack visibility into how a seemingly small $500/mo lifestyle inflation today drastically pushes out their FIRE date and increases their final FIRE target number. This feature solves the problem of "invisible wealth erosion" by making the long-term impact of lifestyle creep tangible.

## 👤 User Story
"As a professional experiencing income growth, I want to model how incremental lifestyle inflation affects my FIRE timeline, so that I can make informed, conscious decisions about increasing my monthly burn rate."

## 🎯 Metric Definition
- **Success:** Users can visualize the exact delay (in months/years) added to their FIRE date per $100 of lifestyle creep.
- **Engagement:** 20% of users who run a FIRE simulation also run a Lifestyle Creep scenario.

## ⚖️ Gap Analysis
- **Current State:** Most standard retirement calculators assume a static monthly expense rate and require manual, separate runs to compare different expense levels.
- **Market Gap:** We will provide a native, integrated curve showing the correlation between expense inflation and FIRE delay.

## ✅ Acceptance Criteria
- Must take a base FIRE configuration and a projected annual "creep" percentage or fixed amount.
- Must output a comparative timeline showing "Time to FIRE (Static)" vs "Time to FIRE (With Creep)".
- Must clearly display the difference in the final FIRE target number.
- Must remain purely domain-driven without coupling to specific UI layers.

## 🚫 Out of Scope
- Granular category-level creep modeling (e.g., separating housing vs. dining inflation).
- Automatic historical trend detection of creep (Phase 2).
