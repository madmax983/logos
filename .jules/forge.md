**[Refactoring Command Line Parsers]
**Learning:** Argument parsing in Rust often leads to deeply nested `.map().filter()` chains if not careful. The CLI arg parsers used heavy chaining (`parse_optional_flag_value(args, flag)?.map_or(Ok(default_value), |value| { value.parse::<i64>().map_err(...) })`).
**Action:** Replace `Result::map_err` or `Option::map_or` inside large functions with flat "Guard Clauses" (`let Some(value) = ... else { return Ok(...) }`) to flatten nesting, improving readability significantly.
