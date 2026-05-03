import re

with open('crates/logos-cli/src/args.rs', 'r') as f:
    content = f.read()

# Let's add the new command Plan and subcommand Fire
