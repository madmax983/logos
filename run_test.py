import subprocess

def run_cmd(cmd):
    return subprocess.run(cmd, shell=True, text=True, capture_output=True)

with open('crates/logos-tui/tests/reconcile_screen.rs', 'r') as f:
    text = f.read()

# Let's print out the frame to see what it actually looks like.
text = text.replace('assert!(frame.contains("selected_run=recon-42"));', 'print!("FRAME: {}", frame); assert!(frame.contains("selected_run=recon-42"));')
text = text.replace('assert!(first_frame.contains("selected_run=recon-1"));', 'print!("FRAME1: {}", first_frame); assert!(first_frame.contains("selected_run=recon-1"));')

with open('crates/logos-tui/tests/reconcile_screen.rs', 'w') as f:
    f.write(text)

res = run_cmd('cargo test -p logos-tui --test reconcile_screen -- --nocapture')
print(res.stdout)

# Restore the file
run_cmd('git restore crates/logos-tui/tests/reconcile_screen.rs')
