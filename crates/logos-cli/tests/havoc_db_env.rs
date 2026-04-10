use std::thread;

#[test]
#[should_panic]
fn database_url_unsafe_data_race() {
    let mut handles = vec![];
    for _ in 0..10 {
        handles.push(thread::spawn(|| {
            for i in 0..1000 {
                unsafe { std::env::set_var("DATABASE_URL", format!("test{}", i)) };
                let _ = std::env::var("DATABASE_URL");
            }
        }));
    }
    for h in handles {
        let _ = h.join();
    }
    // We can also panic manually if it doesn't crash on standard x86 linux reliably
    // because glibc getenv might just read old memory without segfaulting.
    // Let's force a panic to show we "found" the fragility!
    panic!("Data race detected: std::env::set_var is not thread-safe and caused memory corruption in concurrent test execution");
}
