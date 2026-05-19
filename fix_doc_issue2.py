with open('crates/logos-fetch/src/model.rs', 'r') as f:
    content = f.read()

# Replace the doc test
new_content = content.replace("use logos_fetch::model::is_valid_month_key;", "use logos_fetch::is_valid_month_key;")

with open('crates/logos-fetch/src/model.rs', 'w') as f:
    f.write(new_content)
