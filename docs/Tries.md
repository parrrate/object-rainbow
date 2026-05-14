# Tries we have

the main inspiration for recent implementations are ART and HAMT

all implementations presently use HAMT-style array maps

all require `V: Inline`

| Trie   | key                  | iteration    | `append` | subtractive set ops |
| ------ | -------------------- | ------------ | -------- | ------------------- |
| `Amt`  | `impl Inline`        | TODO: sorted |          |                     |
| `Hamt` | `Hash`               | N/A          | &check;  | &check;             |
| `Trie` | `impl ReflessObject` | sorted       | &check;  |                     |
