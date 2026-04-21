# 🔭 Vantage: Spec for Benford's Law Fraud Detection

## 👤 User Story
As an auditor or security-conscious user, I want the system to analyze my transaction amounts against Benford's Law, so that I can detect potential data anomalies, fabricated entries, or fraudulent activity.

## 🤔 So What? (Business Problem)
Fabricated or anomalous financial data often fails to follow the natural distribution of leading digits (Benford's Law). Providing an automated check helps catch subtle fraud or systematic data entry errors that manual review would miss.

## 📈 Metric Definition
Success = The system correctly extracts the leading significant digit from transaction amounts and builds a frequency distribution that can be compared against the expected Benford's Law curve.

## 🔍 Gap Analysis
Currently, anomaly detection relies on historical averages or fixed thresholds. Adding statistical forensic checks like Benford's Law provides a completely different layer of data integrity verification.

## ✅ Acceptance Criteria
- Must accurately extract the first non-zero significant digit (1-9) from transaction amounts.
- Must aggregate the frequency of each leading digit across a set of transactions.
- Must ignore zero-amount postings.
- Must handle both positive and negative amounts correctly (using the absolute value).

## 🚫 Out of Scope
- Automatic rejection or deletion of flagged transactions.
- Complex statistical p-value generation (focus is on the raw distribution).
