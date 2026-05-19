import os
import re

def check_file(path):
    if 'tests/' in path: return
    with open(path, 'r') as f:
        content = f.read()

    if path.endswith('/lib.rs') or path.endswith('/mod.rs'):
        if not content.startswith('//!'):
            print(f"Missing module doc in {path}")

for root, _, files in os.walk('crates'):
    for file in files:
        if file.endswith('.rs'):
            check_file(os.path.join(root, file))
