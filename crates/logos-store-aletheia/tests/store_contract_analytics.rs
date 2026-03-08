use logos_store_aletheia::{AletheiaStore, StoreError};

#[test]
fn write_analytics_artifact_us_forwards_to_main_method() {
    let mut store = AletheiaStore::new();
    let manifest = store
        .write_analytics_artifact_manifest_us(
            "csv",
            "uri",
            "hash",
            1,
            1,
            1_000_000_000,
            2_000_000_000,
            None,
        )
        .unwrap();

    assert_eq!(manifest.artifact_kind(), "csv");
    assert_eq!(manifest.snapshot_valid_at().wallclock(), 1_000_000_000);
    assert_eq!(manifest.snapshot_tx_at().wallclock(), 2_000_000_000);
}

#[test]
fn write_analytics_artifact_fails_when_supersedes_id_is_unknown() {
    let mut store = AletheiaStore::new();
    let err = store
        .write_analytics_artifact_manifest(
            "parquet",
            "uri",
            "hash",
            1,
            1,
            aletheiadb::time::now(),
            aletheiadb::time::now(),
            Some("unknown-id"),
        )
        .unwrap_err();

    assert!(matches!(err, StoreError::UnknownArtifact { .. }));
}
