set -e
cargo hack clippy --package object-rainbow --feature-powerset --group-features bitvec,bytes,cid,indexmap,ulid --lib --tests --bins
cargo hack clippy --workspace --feature-powerset --lib --tests --bins --exclude object-rainbow --exclude object-rainbow-schema --exclude xtask  --exclude xtask-release
cargo hack clippy --package object-rainbow-schema --feature-powerset --exclude-features _collections --lib --tests --bins
