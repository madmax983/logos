import re

with open('crates/logos-core/src/experimental/fire_goal_seeker.rs', 'r') as f:
    content = f.read()

content = content.replace("5_000_000_00", "500_000_000")

with open('crates/logos-core/src/experimental/fire_goal_seeker.rs', 'w') as f:
    f.write(content)
