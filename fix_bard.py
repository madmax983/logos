import sys

with open('.jules/bard.md', 'r') as f:
    lines = f.readlines()

new_lines = []
for line in lines:
    if line.startswith('## 2025-05-03 - The "Black Box" of TUI Views'):
        new_lines.extend([
            '## 2025-05-03 - The "Black Box" of TUI Views\n',
            '**Confusion:** The `logos-tui/src/ui/mod.rs` file was an undocumented black box that simply re-exported modules. Developers couldn\'t tell that these modules contained pure functions designed to be used independently of ratatui, leading to potential confusion about how views are rendered.\n',
            '**Clarification:** Added module-level `//!` documentation explaining the pure-function architecture of the `ui` module, including an ignored code example demonstrating how to render a view without any terminal setup.\n'
        ])
    elif line.startswith('**Confusion:** The `logos-tui/src/ui/mod.rs`') or line.startswith('**Clarification:** Added module-level `//!`'):
        pass
    else:
        new_lines.append(line)

with open('.jules/bard.md', 'w') as f:
    f.writelines(new_lines)
