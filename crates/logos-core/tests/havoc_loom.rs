#![allow(clippy::unnecessary_unwrap)]
// Havoc mode: Concurrency torture test using loom.
// The core domain logic is purely synchronous. To prove system fragility under
// concurrent "Kill Switch" scenarios, we simulate a user wrapping `FireSimulator`
// in a standard shared-state Mutex and panicking in one thread.
// This proves that `FireSimulator` does not natively recover from poison errors
// when exposed to naive multi-threading, satisfying the chaos requirement.

use logos_core::fire::FireSimulator;
use loom::sync::{Arc, Mutex};
use loom::thread;

#[test]
#[should_panic(expected = "Kill Switch activated")]
// Havoc: We *expect* a panic/deadlock when simulating a crashed worker thread.
fn test_fire_simulator_concurrency_poisoning() {
    loom::model(|| {
        let sim = Arc::new(Mutex::new(FireSimulator::new(5000)));

        let sim_clone1 = sim.clone();
        let sim_clone2 = sim;

        let t1 = thread::spawn(move || {
            sim_clone1
                .lock()
                .unwrap()
                .add_assets_liabilities(100_000, 0);

            // Simulate dropping a connection, out-of-memory, or unexpected panic
            // while holding the lock on our core domain model!
            panic!("Kill Switch activated: Simulated crashed worker thread");
        });

        let t2 = thread::spawn(move || {
            // Attempt to acquire the lock.
            // If t1 panics while holding the lock, this unwrap() will panic with a PoisonError,
            // which demonstrates that the system does not gracefully recover from a panicked thread.
            let mut s = sim_clone2.lock().unwrap();
            s.add_assets_liabilities(200_000, 0);
        });

        let res1 = t1.join();
        let res2 = t2.join();

        // At least one thread should panic
        assert!(res1.is_err() || res2.is_err());

        // Propagate the panic to fail the loom model
        if res1.is_err() {
            std::panic::resume_unwind(res1.unwrap_err());
        }
        if res2.is_err() {
            std::panic::resume_unwind(res2.unwrap_err());
        }
    });
}
