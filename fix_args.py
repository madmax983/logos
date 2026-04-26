with open("crates/logos-cli/src/args.rs", "r") as f:
    content = f.read()

content = content.replace("multiplier: f64", "multiplier: String")
content = content.replace("anomaly_detect(*multiplier)", "anomaly_detect(multiplier.parse::<f64>().unwrap_or(1.5))")
content = content.replace("""let multiplier = parse_optional_parsed_flag::<f64>(&args[2..], "--multiplier", 1.5)?;""", """let multiplier = parse_optional_flag_value(&args[2..], "--multiplier")?.unwrap_or_else(|| "1.5".to_string());""")

content = content.replace("#[derive(Debug, Clone, PartialEq)]\npub enum AnalyticsCommand", "#[derive(Debug, Clone, PartialEq, Eq)]\npub enum AnalyticsCommand")

with open("crates/logos-cli/src/args.rs", "w") as f:
    f.write(content)
