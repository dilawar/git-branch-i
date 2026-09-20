lint:
  cargo clippy -- -Dwarnings

fmt:
  cargo +nightly fmt

fix:
  cargo clippy --fix --allow-dirty
