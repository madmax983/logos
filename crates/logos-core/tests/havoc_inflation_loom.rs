#![allow(clippy::unnecessary_unwrap)]
#[cfg(feature = "nova")]
use logos_core::inflation::InflationProjector;
#[cfg(feature = "nova")]
use loom::sync::{Arc, Mutex};
#[cfg(feature = "nova")]
use loom::thread;

#[cfg(feature = "nova")]
#[test]
#[should_panic(expected = "Kill Switch activated")]
fn test_inflation_projector_concurrency_poisoning() {
    loom::model(|| {
        let sim = Arc::new(Mutex::new(InflationProjector::new(3.0)));

        let sim_clone1 = sim.clone();
        let sim_clone2 = sim;

        let t1 = thread::spawn(move || {
            let _guard = sim_clone1.lock().unwrap();
            let _ = _guard.future_nominal_cost_cents(100_000, 10);
            panic!("Kill Switch activated: Simulated crashed worker thread");
        });

        let t2 = thread::spawn(move || {
            let _guard = sim_clone2.lock().unwrap();
            let _ = _guard.future_nominal_cost_cents(200_000, 10);
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
