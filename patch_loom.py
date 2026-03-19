with open("crates/logos-core/tests/havoc_loom.rs", "r") as f:
    content = f.read()

# Replace #[should_panic]
content = content.replace("#[should_panic] // Havoc: We *expect*", '#[should_panic(expected = "Kill Switch activated")]\n// Havoc: We *expect*')

# Fix unwrap_err
content = content.replace("if res1.is_err() { std::panic::resume_unwind(res1.unwrap_err()); }", "if let Err(e) = res1 { std::panic::resume_unwind(e); }")
content = content.replace("if res2.is_err() { std::panic::resume_unwind(res2.unwrap_err()); }", "if let Err(e) = res2 { std::panic::resume_unwind(e); }")

# Remove redundant clone
content = content.replace("let sim_clone2 = sim.clone();", "let sim_clone2 = sim;")

# Fix early drop
content = content.replace("let mut s = sim_clone1.lock().unwrap();\n            s.add_assets_liabilities(100_000, 0);", "sim_clone1.lock().unwrap().add_assets_liabilities(100_000, 0);")

with open("crates/logos-core/tests/havoc_loom.rs", "w") as f:
    f.write(content)

print("Loom patched!")
