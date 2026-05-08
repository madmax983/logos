import os
import re

def process_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # match function definitions
    pattern = re.compile(r'^\s*(?:pub(?:\([^)]+\))?\s+)?(?:async\s+)?fn\s+([a-zA-Z0-9_]+)\s*(?:<[^>]+>)?\s*\(', re.MULTILINE)

    lines = content.split('\n')
    funcs = []

    in_func = False
    func_name = ""
    start_line = 0
    brace_count = 0

    for i, line in enumerate(lines):
        if not in_func:
            m = pattern.match(line)
            if m:
                in_func = True
                func_name = m.group(1)
                start_line = i
                brace_count = line.count('{') - line.count('}')
        else:
            brace_count += line.count('{') - line.count('}')
            if brace_count == 0:
                in_func = False
                length = i - start_line
                if length > 50:
                    funcs.append((func_name, length, start_line + 1))

    return funcs

results = []
for root, _, files in os.walk('crates'):
    for file in files:
        if file.endswith('.rs'):
            filepath = os.path.join(root, file)
            funcs = process_file(filepath)
            for func in funcs:
                results.append((filepath, func[0], func[1], func[2]))

results.sort(key=lambda x: x[2], reverse=True)
for r in results[:40]:
    print(f"{r[0]}:{r[3]} - {r[1]} ({r[2]} lines)")
