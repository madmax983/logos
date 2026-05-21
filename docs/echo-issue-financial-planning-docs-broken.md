# 🗣️ Echo: Getting Started example is broken

## 🤦 The Confusion
I am a new user trying to use the financial planning primitives. I ran the walkthrough script `run_echo_test.sh` which copies the example into a fresh `main.rs`. When I tried to run it, the compiler blew up with:
`error[E0603]: module 'planning' is private`
I couldn't even get the first line to compile! If the example doesn't compile out of the box, I am leaving.

## 🕵️ The Reality
Turns out the `planning` module in `logos_core` is private (`pub(crate)`). The script is using the internal module paths (e.g., `use logos_core::planning::fire::{FireSimulator, UpcomingVest};`) instead of the public re-exports at the root of the crate.

## 💡 The Fix
Update the `use` statements in the example and documentation to point to the correct public paths.
For example, change:
`use logos_core::planning::fire::{FireSimulator, UpcomingVest};`
to:
`use logos_core::fire::{FireSimulator, UpcomingVest};`