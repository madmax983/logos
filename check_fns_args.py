import re
import os

def check_file(path):
    if 'tests/' in path: return
    with open(path, 'r') as f:
        content = f.read()

    # check if 'pub fn' is missing docs or if missing '## Examples' block if it is a public item on a public module
    # wait let's just use what I have. I am missing `pub struct PostgresStore` doc in `logos-store-pg` and `pub fn is_valid_month_key` in `logos-fetch/src/model.rs`
