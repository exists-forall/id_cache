# `id_cache`

**Download:** [crates.io/crates/id_cache](https://crates.io/crates/id_cache)

**Docs:** [docs.rs/id_cache](https://docs.rs/id_cache)

---

This is a small crate built on [`id_collections`](https://crates.io/crates/id_collections) which provides a simple "cache" data structure for generating sequentially-assigned ids for unique values of some hashable type.

## Example

```rust
use id_collections::id_type;
use id_cache::IdCache;

#[id_type]
struct WordId(u32);

let mut word_cache: IdCache<WordId, &str> = IdCache::new();

let foo_id = word_cache.make_id("foo");
let bar_id = word_cache.make_id("bar");

assert_eq!(word_cache[foo_id], "foo");
assert_eq!(word_cache[bar_id], "bar");

// ids for repeated values are reused:
assert_eq!(word_cache.make_id("foo"), foo_id);
```

## Optional Features

This crate has optional [Serde](https://serde.rs) support, which can be enabled with the `serde` Cargo feature.
