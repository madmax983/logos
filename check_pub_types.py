import re
import os

def check_file(path):
    if 'tests/' in path: return
    with open(path, 'r') as f:
        content = f.read()

    lines = content.split('\n')
    for i, line in enumerate(lines):
        match = re.search(r'^\s*(pub\s+)?(struct|enum|fn)\s+([A-Z][a-zA-Z0-9_]*|[a-z0-9_]+)\s*(\(|{|<|;)', line)
        if match:
            # We want to check if it's public
            if 'pub ' not in line:
                continue
            if 'pub(crate)' in line or 'pub(super)' in line:
                continue

            name = match.group(3)

            # Check for Examples section
            has_examples = False
            # Look backwards from the item for doc comments
            j = i - 1
            docs = []
            while j >= 0 and ('///' in lines[j] or '#[' in lines[j]):
                if '///' in lines[j]:
                    docs.append(lines[j])
                j -= 1
            docs.reverse()
            full_doc = '\n'.join(docs)
            if '## Examples' not in full_doc and 'Examples' not in full_doc and '## Example' not in full_doc:
                print(f"{path}:{i+1} missing Examples for: {line.strip()}")

for root, _, files in os.walk('crates'):
    for file in files:
        if file.endswith('.rs'):
            check_file(os.path.join(root, file))
