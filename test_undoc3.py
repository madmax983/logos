import re
import os

def check_file(path):
    with open(path, 'r') as f:
        content = f.read()

    lines = content.split('\n')
    for i, line in enumerate(lines):
        match = re.search(r'^\s*pub\s+(struct|enum|fn)\s+([A-Z][a-zA-Z0-9_]*)', line)
        if match:
            if i > 0 and '///' not in lines[i-1] and '#[' not in lines[i-1] and '{' not in lines[i-1]:
                print(f"{path}:{i+1} missing doc: {line}")

for root, _, files in os.walk('crates'):
    for file in files:
        if file.endswith('.rs'):
            check_file(os.path.join(root, file))
