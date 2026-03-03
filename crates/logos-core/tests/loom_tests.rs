use loom::sync::{Arc, Mutex};
use loom::thread;

#[test]
fn loom_test_concurrent_rollover() {
    loom::model(|| {
        let budget = Arc::new(Mutex::new(100_000));

        let t1_budget = budget.clone();
        let t1 = thread::spawn(move || {
            let mut val = t1_budget.lock().unwrap();
            *val = logos_core::rollover_end_balance(*val, 50_000, 25_000);
        });

        let t2_budget = budget.clone();
        let t2 = thread::spawn(move || {
            let mut val = t2_budget.lock().unwrap();
            *val = logos_core::rollover_end_balance(*val, 10_000, 5_000);
        });

        t1.join().unwrap();
        t2.join().unwrap();

        let final_balance = *budget.lock().unwrap();
        assert_eq!(final_balance, 130_000);
    });
}
