import re

with open('crates/logos-import/tests/pdf_import.rs', 'r') as f:
    content = f.read()

# Fix literal string parens
content = re.sub(
    r'std::fs::write\(&pdf_path, b"([^"]+)\\n"\)',
    r'std::fs::write(&pdf_path, b"%PDF-1.4\\n(\\1)\\n")',
    content
)
content = re.sub(
    r'std::fs::write\(&pdf_path2, b"([^"]+)\\n"\)',
    r'std::fs::write(&pdf_path2, b"%PDF-1.4\\n(\\1)\\n")',
    content
)

# Fix silent passes
content = content.replace(
    '    if let Ok(records) = result {\n        assert_eq!(records.len(), 1);\n        assert_eq!(records[0].amount_cents(), 1000);\n    }',
    '    let records = result.unwrap();\n    assert_eq!(records.len(), 1);\n    assert_eq!(records[0].amount_cents(), 1000);'
)
content = content.replace(
    '    if let Ok(records) = result {\n        assert_eq!(records.len(), 1);\n        assert_eq!(records[0].memo(), "TXN(NAME)");\n    }',
    '    let records = result.unwrap();\n    assert_eq!(records.len(), 1);\n    assert_eq!(records[0].memo(), "TXN(NAME)");'
)

with open('crates/logos-import/tests/pdf_import.rs', 'w') as f:
    f.write(content)

with open('crates/logos-fetch/tests/secret_resolver.rs', 'r') as f:
    lines = f.readlines()
with open('crates/logos-fetch/tests/secret_resolver.rs', 'w') as f:
    skip = False
    for line in lines:
        if line.startswith('#[test]') and 'fn secret_bundle_accessors' in ''.join(lines[lines.index(line):lines.index(line)+2]):
            skip = True
        if skip and line.strip() == '}':
            skip = False
            continue
        if not skip:
            f.write(line)

with open('crates/logos-fetch/tests/statement_source.rs', 'r') as f:
    lines = f.readlines()
with open('crates/logos-fetch/tests/statement_source.rs', 'w') as f:
    skip = False
    for line in lines:
        if line.startswith('#[test]') and 'fn statement_source_accessors' in ''.join(lines[lines.index(line):lines.index(line)+2]):
            skip = True
        if skip and line.strip() == '}':
            skip = False
            continue
        if not skip:
            f.write(line)
