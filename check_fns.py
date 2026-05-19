import re
import os

def check_file(path):
    if 'tests/' in path: return
    with open(path, 'r') as f:
        content = f.read()

    lines = content.split('\n')
    for i, line in enumerate(lines):
        match = re.search(r'^\s*pub\s+fn\s+([a-z0-9_]+)\s*\(', line)
        if match:
            if i > 0 and '///' not in lines[i-1] and '#[' not in lines[i-1]:
                print(f"{path}:{i+1} missing doc: {line}")

for root, _, files in os.walk('crates'):
    for file in files:
        if file.endswith('.rs'):
            check_file(os.path.join(root, file))
