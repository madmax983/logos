import os
import re

def check_docs(directory):
    for root, dirs, files in os.walk(directory):
        for file in files:
            if not file.endswith('.rs'): continue
            path = os.path.join(root, file)
            with open(path, 'r') as f:
                content = f.read()

                lines = content.split('\n')
                for i, line in enumerate(lines):
                    if line.strip() == "pub struct PostgresStore {":
                        print(f"PostgresStore found in {path}:{i+1}")
                        if i > 0:
                            print(f"Previous line: {lines[i-1]}")

check_docs('crates')
