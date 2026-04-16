cat << 'INNER_EOF' | patch -p0
--- crates/logos-store/src/model.rs
+++ crates/logos-store/src/model.rs
@@ -642,3 +642,86 @@
         self.created_at
     }
 }
+
+#[cfg(test)]
+mod tests2 {
+    use super::*;
+
+    #[test]
+    fn test_stored_reconciliation_run_getters() {
+        let ts: Timestamp = 1_696_118_400;
+        let run = StoredReconciliationRun::new(
+            "run_123",
+            "2023-10",
+            "acct_checking",
+            1000,
+            500,
+            1500,
+            1500,
+            0,
+            true,
+            10,
+            5,
+            800,
+            300,
+            ts,
+        );
+
+        assert_eq!(run.run_id(), "run_123");
+        assert_eq!(run.month_key(), "2023-10");
+        assert_eq!(run.checking_account(), "acct_checking");
+        assert_eq!(run.opening_balance_cents(), 1000);
+        assert_eq!(run.ledger_delta_cents(), 500);
+        assert_eq!(run.expected_closing_balance_cents(), 1500);
+        assert_eq!(run.statement_closing_balance_cents(), 1500);
+        assert_eq!(run.variance_cents(), 0);
+        assert_eq!(run.reconciled(), true);
+        assert_eq!(run.matched_postings(), 10);
+        assert_eq!(run.matched_transaction_count(), 5);
+        assert_eq!(run.inflow_cents(), 800);
+        assert_eq!(run.outflow_cents(), 300);
+        assert_eq!(run.created_at(), ts);
+    }
+
+    #[test]
+    fn test_stored_month_close_getters() {
+        let ts: Timestamp = 1_696_118_400;
+        let mc = StoredMonthClose::new(
+            "close_123",
+            "2023-10",
+            "acct_checking",
+            "run_123",
+            Some("art_123"),
+            ts,
+        );
+
+        assert_eq!(mc.close_id(), "close_123");
+        assert_eq!(mc.month_key(), "2023-10");
+        assert_eq!(mc.checking_account(), "acct_checking");
+        assert_eq!(mc.reconciliation_run_id(), "run_123");
+        assert_eq!(mc.analytics_artifact_id(), Some("art_123"));
+        assert_eq!(mc.closed_at(), ts);
+    }
+
+    #[test]
+    fn test_stored_statement_line_getters() {
+        let ts: Timestamp = 1_696_118_400;
+        let txn_id = TransactionId::new("txn_123").unwrap();
+        let stmt = StoredStatementLine::new(
+            "stmt_123",
+            "batch_123",
+            "source_123",
+            "2023-10-01",
+            "memo",
+            1000,
+            Some(txn_id.clone()),
+        );
+
+        assert_eq!(stmt.statement_line_id(), "stmt_123");
+        assert_eq!(stmt.batch_id(), "batch_123");
+        assert_eq!(stmt.source_uri(), "source_123");
+        assert_eq!(stmt.statement_timestamp(), "2023-10-01");
+        assert_eq!(stmt.memo(), "memo");
+        assert_eq!(stmt.amount_cents(), 1000);
+        assert_eq!(stmt.imported_txn_id(), Some(&txn_id));
+    }
+
+    #[test]
+    fn test_stored_import_batch_getters() {
+        let ts: Timestamp = 1_696_118_400;
+        let batch = StoredImportBatch::new(
+            "batch_123",
+            "import_kind",
+            "source_uri",
+            "batch_key",
+            10,
+            5,
+            true,
+            false,
+            ts,
+        );
+
+        assert_eq!(batch.batch_id(), "batch_123");
+        assert_eq!(batch.import_kind(), "import_kind");
+        assert_eq!(batch.source_uri(), "source_uri");
+        assert_eq!(batch.batch_key(), "batch_key");
+        assert_eq!(batch.record_count(), 10);
+        assert_eq!(batch.duplicate_count(), 5);
+        assert_eq!(batch.dry_run(), true);
+        assert_eq!(batch.ocr_enabled(), false);
+        assert_eq!(batch.created_at(), ts);
+    }
+
+    #[test]
+    fn test_new_statement_line_getters() {
+        let ts: Timestamp = 1_696_118_400;
+        let line = NewStatementLine::new(
+            "source_uri",
+            "2023-10-01",
+            "memo",
+            1000,
+        );
+
+        assert_eq!(line.source_uri(), "source_uri");
+        assert_eq!(line.statement_timestamp(), "2023-10-01");
+        assert_eq!(line.memo(), "memo");
+        assert_eq!(line.amount_cents(), 1000);
+    }
+
+    #[test]
+    fn test_stored_import_record_getters() {
+        let ts: Timestamp = 1_696_118_400;
+        let txn_id = TransactionId::new("txn_123").unwrap();
+        let rec = StoredImportRecord::new(
+            "rec_123",
+            "batch_123",
+            Some(txn_id.clone()),
+            ts,
+        );
+
+        assert_eq!(rec.record_id(), "rec_123");
+        assert_eq!(rec.batch_id(), "batch_123");
+        assert_eq!(rec.imported_txn_id(), Some(&txn_id));
+        assert_eq!(rec.created_at(), ts);
+    }
+}
INNER_EOF
