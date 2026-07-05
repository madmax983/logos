with open("crates/logos-cli/src/args.rs", "r") as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if "pub enum Command {" in line:
        print(f"Command found at line {i}")
    if "fn parse_args_impl(" in line:
        print(f"parse_args_impl found at line {i}")
