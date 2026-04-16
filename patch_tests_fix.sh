cat << 'INNER_EOF' | patch -p0
--- crates/logos-store/src/model.rs
+++ crates/logos-store/src/model.rs
@@ -1048,11 +1048,12 @@
             "2023-10-01",
             "memo",
             1000,
             Some(txn_id.clone()),
+            ts,
         );

-        assert_eq!(stmt.statement_line_id(), "stmt_123");
+        assert_eq!(stmt.statement_id(), "stmt_123");
         assert_eq!(stmt.batch_id(), "batch_123");
         assert_eq!(stmt.source_uri(), "source_123");
         assert_eq!(stmt.statement_timestamp(), "2023-10-01");
         assert_eq!(stmt.memo(), "memo");
INNER_EOF
