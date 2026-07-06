import subprocess
import os

crates = [
    "logos-cli",
    "logos-core",
    "logos-fetch",
    "logos-import",
    "logos-reporting",
    "logos-runtime",
    "logos-store",
    "logos-store-pg",
    "logos-tui"
]

for crate in crates:
    print(f"Scanning {crate}...")
    try:
        # Check if mutants are actually found
        output = subprocess.check_output(f"cargo mutants -p {crate} --timeout 60", shell=True, text=True, stderr=subprocess.STDOUT)
        if "missed" in output.lower() or "survived" in output.lower():
            print(f"Found survivors in {crate}!")
            print(output)
        else:
            print(f"No survivors in {crate}.")
    except subprocess.CalledProcessError as e:
        print(f"Failed to scan {crate} or survivors found. Output:")
        print(e.output)
