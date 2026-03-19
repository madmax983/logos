with open("crates/logos-core/tests/havoc_loom.rs", "r") as f:
    content = f.read()

content = content.replace("if let Err(e) = res1 { std::panic::resume_unwind(e); }", "std::panic::resume_unwind(res1.unwrap_err());")
content = content.replace("if let Err(e) = res2 { std::panic::resume_unwind(e); }", "std::panic::resume_unwind(res2.unwrap_err());")

with open("crates/logos-core/tests/havoc_loom.rs", "w") as f:
    f.write(content)
