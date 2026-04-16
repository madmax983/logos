cat << 'INNER_EOF' | patch -p0
--- crates/logos-store/src/model.rs
+++ crates/logos-store/src/model.rs
@@ -1054,7 +1054,7 @@
             ts,
         );

-        assert_eq!(stmt.statement_id(), "stmt_123");
+        assert_eq!(stmt.line_id(), "stmt_123");
         assert_eq!(stmt.batch_id(), "batch_123");
         assert_eq!(stmt.source_uri(), "source_123");
         assert_eq!(stmt.statement_timestamp(), "2023-10-01");
INNER_EOF
