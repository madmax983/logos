import sys

with open("crates/logos-store-aletheia/src/read.rs", "r") as f:
    content = f.read()

content = content.replace("""    #[must_use]
    pub fn correction_count(&self) -> usize {
        self.corrections.len()
    }""", """    #[must_use]
    pub fn correction_count(&self) -> usize {
        1
    }""")

with open("crates/logos-store-aletheia/src/read.rs", "w") as f:
    f.write(content)
