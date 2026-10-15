# run the editor in dev mode
run:
    cargo run -p noetic_editor

# apply clippy fixes and format the code
fix:
    cargo clippy --workspace --all-targets --fix --allow-dirty
    cargo fmt --all

# check formatting and lints (tests included), and run the tests
check:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

# run the tests only
test:
    cargo test --workspace
