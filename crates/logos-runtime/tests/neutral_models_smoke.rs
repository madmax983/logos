use logos_runtime::{MonthAutopilotSummary, MonthReport};
use logos_store::model::{
    StoredFetchArtifactFormat, StoredFetchRun, StoredFetchRunStatus, StoredMonthClose,
    StoredReconciliationRun,
};

#[test]
fn month_autopilot_summary_accepts_neutral_store_models() {
    let summary = MonthAutopilotSummary::new(
        "2026-03",
        "assets:checking",
        2,
        1,
        vec![StoredFetchRun::new(
            "fetch-1",
            "source-1",
            "bank-1",
            "assets:checking",
            "2026-03",
            StoredFetchRunStatus::Downloaded,
            Some("/tmp/fetch.csv"),
            Some(StoredFetchArtifactFormat::Csv),
            Some(100),
            Some(200),
            None,
            42,
        )],
        StoredReconciliationRun::new(
            "recon-1",
            "2026-03",
            "assets:checking",
            1_000,
            250,
            1_250,
            1_250,
            0,
            true,
            3,
            2,
            500,
            250,
            99,
        ),
        MonthReport::new(1_250, 2_000, 750, 1_250),
        StoredMonthClose::new(
            "close-1",
            "2026-03",
            "assets:checking",
            "recon-1",
            None,
            100,
        ),
    );

    assert_eq!(summary.month_key(), "2026-03");
    assert_eq!(summary.fetch_runs().len(), 1);
    assert_eq!(summary.reconciliation_run().run_id(), "recon-1");
    assert_eq!(summary.close().close_id(), "close-1");
}
