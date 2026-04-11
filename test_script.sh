#!/bin/bash
cat crates/logos-core/src/experimental/recurrence_detector.rs | grep -A 40 "pub fn detect"
