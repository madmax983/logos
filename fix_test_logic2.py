import re

with open('crates/logos-import/tests/pdf_import.rs', 'r') as f:
    content = f.read()

# Fix literal string parens manually without bad regex references
content = re.sub(r'b"%PDF-1\.4\\n\(\\1\)\\n"', r'b"%PDF-1.4\n(2026-02-01 ITEM 10.00 -50.00)\n"', content, count=1)
content = re.sub(r'b"%PDF-1\.4\\n\(\\1\)\\n"', r'b"%PDF-1.4\n(2026-02-01 ITEM 10.00 50.00)\n"', content, count=1)
content = re.sub(r'b"%PDF-1\.4\\n\(\\1\)\\n"', r'b"%PDF-1.4\n(02/01/26 ITEM 10.00)\n"', content, count=1)
content = re.sub(r'b"%PDF-1\.4\\n\(\\1\)\\n"', r'b"%PDF-1.4\n(02/01/2026 ITEM 10.00)\n"', content, count=1)
content = re.sub(r'b"%PDF-1\.4\\n\(\\1\)\\n"', r'b"%PDF-1.4\n(0000-01-01 ITEM 10.00\n2026-13-01 ITEM 10.00\n2026-01-32 ITEM 10.00\n2026-04-31 ITEM 10.00)\n"', content, count=1)
content = re.sub(r'b"%PDF-1\.4\\n\(\\1\)\\n"', r'b"%PDF-1.4\n(2024-02-29 ITEM 10.00\n2026-02-29 ITEM 10.00\n1900-02-29 ITEM 10.00\n2000-02-29 ITEM 10.00)\n"', content, count=1)
content = re.sub(r'b"%PDF-1\.4\\n\(\\1\)\\n"', r'b"%PDF-1.4\n(2026-02-01 ITEM 10\n2026-02-02 ITEM 10.\n2026-02-03 ITEM 10.1\n2026-02-04 ITEM 10.12\n2026-02-05 ITEM 10.123)\n"', content, count=1)
content = re.sub(r'b"%PDF-1\.4\\n\(\\1\)\\n"', r'b"%PDF-1.4\n((2026-02-01 TXN 10.00))\n"', content, count=1)
content = re.sub(r'b"%PDF-1\.4\\n\(\\1\)\\n"', r'b"%PDF-1.4\n(NOTHING USEFUL HERE)\n"', content, count=1)
content = re.sub(r'b"%PDF-1\.4\\n\(\\1\)\\n"', r'b"%PDF-1.4\n((2026-02-01 TXN\\(NAME\\) 10.00))\n"', content, count=1)


with open('crates/logos-import/tests/pdf_import.rs', 'w') as f:
    f.write(content)
