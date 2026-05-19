import re
import os

def check_file(path):
    if 'tests/' in path: return
    with open(path, 'r') as f:
        content = f.read()

    lines = content.split('\n')
    for i, line in enumerate(lines):
        match = re.search(r'^\s*(pub\s+)?(struct|enum|fn)\s+([A-Z][a-zA-Z0-9_]*|[a-z0-9_]+)\s*(\(|{|<)', line)
        if match:
            # We want to check if it's public
            if 'pub ' not in line:
                continue
            if 'pub(crate)' in line or 'pub(super)' in line:
                continue

            name = match.group(3)

            # Check for doc comments
            if i > 0 and '///' not in lines[i-1] and '#[' not in lines[i-1]:
                print(f"{path}:{i+1} missing doc: {line.strip()}")

for root, _, files in os.walk('crates'):
    for file in files:
        if file.endswith('.rs'):
            check_file(os.path.join(root, file))
