set -e
mkdir -p target/lcov
cargo llvm-cov --all-features --release --workspace --lcov --output-path target/lcov/lcov.info
