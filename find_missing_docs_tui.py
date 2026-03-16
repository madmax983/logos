import os
import re

def check_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # Find public structs, enums, fns that don't have /// above them
    pub_items = re.finditer(r'(?:^|\n)\s*(pub (?:struct|enum|fn) \w+)', content)

    missing = []
    for match in pub_items:
        start_idx = match.start()
        # Look at the lines before the match
        before = content[:start_idx].strip().split('\n')
        if not before or not before[-1].strip().startswith('///') and not before[-1].strip().startswith('#['):
            missing.append(match.group(1).strip())

    if missing:
        print(f"File: {filepath}")
        for m in missing:
            print(f"  {m}")

for root, _, files in os.walk('crates/logos-tui/src'):
    for file in files:
        if file.endswith('.rs'):
            check_file(os.path.join(root, file))
