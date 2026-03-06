use std::process::Command;

fn main() {
    let output = Command::new("cargo")
        .args(&["test", "e2e_month_autopilot_is_atomic_when_close_reference_is_invalid"])
        .output()
        .expect("Failed to execute cargo test");
    println!("{}", String::from_utf8_lossy(&output.stdout));
}
