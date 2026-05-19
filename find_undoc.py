import os
import re

def check_docs(directory):
    for root, dirs, files in os.walk(directory):
        for file in files:
            if not file.endswith('.rs'): continue
            path = os.path.join(root, file)
            with open(path, 'r') as f:
                content = f.read()

                # check module doc
                if not content.startswith('//!') and 'src/lib.rs' not in path and 'mod.rs' not in path:
                    pass # actually just check structs and enums

                lines = content.split('\n')
                for i, line in enumerate(lines):
                    match = re.search(r'^\s*pub (struct|enum|fn) ([A-Z][a-zA-Z0-9_]*)', line)
                    if match:
                        if i > 0 and '///' not in lines[i-1] and '#[' not in lines[i-1]:
                            print(f"{path}:{i+1} missing doc for {match.group(2)}")

check_docs('crates')
