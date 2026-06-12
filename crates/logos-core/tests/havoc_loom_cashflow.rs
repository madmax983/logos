#![allow(clippy::unnecessary_unwrap)]

use logos_core::cashflow_projector::CashflowProjector;
use loom::sync::{Arc, Mutex};
use loom::thread;

#[test]
#[should_panic(expected = "Kill Switch activated")]
fn test_cashflow_projector_concurrency_poisoning() {
    loom::model(|| {
        let projector = Arc::new(Mutex::new(CashflowProjector::new()));

        let projector_clone1 = projector.clone();
        let projector_clone2 = projector;

        let t1 = thread::spawn(move || {
            let mut s = projector_clone1.lock().unwrap();
            s.set_initial_balance("assets:checking", 100_000);

            panic!("Kill Switch activated: Simulated crashed worker thread");
        });

        let t2 = thread::spawn(move || {
            let mut s = projector_clone2.lock().unwrap();
            s.set_initial_balance("assets:savings", 200_000);
        });

        let res1 = t1.join();
        let res2 = t2.join();

        assert!(res1.is_err() || res2.is_err());

        if res1.is_err() {
            std::panic::resume_unwind(res1.unwrap_err());
        }
        if res2.is_err() {
            std::panic::resume_unwind(res2.unwrap_err());
        }
    });
}
