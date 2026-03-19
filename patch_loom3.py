with open("crates/logos-core/tests/havoc_loom.rs", "r") as f:
    content = f.read()

content = content.replace("std::panic::resume_unwind(res1.unwrap_err());", "if res1.is_err() { std::panic::resume_unwind(res1.unwrap_err()); }")
content = content.replace("std::panic::resume_unwind(res2.unwrap_err());", "if res2.is_err() { std::panic::resume_unwind(res2.unwrap_err()); }")
content = "#![allow(clippy::unnecessary_unwrap)]\n" + content

with open("crates/logos-core/tests/havoc_loom.rs", "w") as f:
    f.write(content)
