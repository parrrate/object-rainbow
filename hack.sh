set -e
cargo hack clippy --package object-rainbow --feature-powerset --group-features bitvec,bytes,cid,indexmap,ulid --lib --tests --bins
cargo hack clippy --workspace --feature-powerset --lib --tests --bins --exclude object-rainbow --exclude xtask  --exclude xtask-release
