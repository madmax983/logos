import re

with open('crates/logos-core/src/experimental/lifestyle_creep.rs', 'r') as f:
    content = f.read()

# Replace hardcoded config with existing config
content = content.replace("current_fire_sim.set_config(FireConfig { safe_withdrawal_rate_pct: 4 });", "current_fire_sim.set_config(*self.fire_sim.config());")

# Also need to add config getter to FireSimulator if it doesn't exist. Let's check first.
