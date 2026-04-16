cat << 'INNER_EOF' | patch -p0
--- crates/logos-reporting/tests/rsu_forecast_havoc.rs
+++ crates/logos-reporting/tests/rsu_forecast_havoc.rs
@@ -9,7 +9,8 @@ proptest! {
         val1 in i64::MAX - 10..=i64::MAX,
         val2 in i64::MAX - 10..=i64::MAX,
     ) {
-        let summary = project_rsu_forecast_summary(&[val1, val2]);
+        let values = [val1, val2];
+        let summary = project_rsu_forecast_summary(&values);
         assert_eq!(summary.projected_total_cents(), i64::MAX);
     }
 }
INNER_EOF
