with open('crates/logos-store-pg/src/store.rs', 'r') as f:
    content = f.read()

new_content = content.replace("/// Concrete PostgreSQL storage implementation.", "/// Concrete `PostgreSQL` storage implementation.")

with open('crates/logos-store-pg/src/store.rs', 'w') as f:
    f.write(new_content)
