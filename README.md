# blazemap

_Implements a vector-based slab-like map with an interface similar to that of `HashMap`,
and also provides tools for generating lightweight identifiers that can be type-safely used as keys for this map._

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/blazemap.svg
[crates-url]: https://crates.io/crates/blazemap
[docs-badge]: https://img.shields.io/docsrs/blazemap
[docs-url]: https://docs.rs/blazemap
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/andrewsonin/blazemap/blob/master/LICENSE
[actions-badge]: https://github.com/andrewsonin/blazemap/actions/workflows/ci.yml/badge.svg
[actions-url]: https://github.com/andrewsonin/blazemap/actions/workflows/ci.yml

## Usage

Currently, this crate provides 3 ways to create new types based on `usize` that can be used as keys to `BlazeMap`.
They are represented by the following macros and provide different optimizations.

### 1. `define_key_wrapper!`

Creates a new type that acts as an `usize`-based replacement
for the old type that can be used as a key for `blazemap` collections.

#### Example

```rust
use blazemap::prelude::{BlazeMap, define_key_wrapper};

define_key_wrapper! {
    pub struct Key(&'static str);
    Derive(as for Original Type): {  // Optional section
        Debug,
        Display,
    };
    Derive(as for usize): {          // Optional section
        Ord,
    }
}

let key_1 = Key::new("first");
let key_2 = Key::new("second");
let key_3 = Key::new("third");

let mut map = BlazeMap::new();
map.insert(key_2, "2");
map.insert(key_1, "1");
map.insert(key_3, "3");

assert_eq!(format!("{map:?}"), r#"{"first": "1", "second": "2", "third": "3"}"#)
```

### 2. `define_key_wrapper_bounded!`

Creates a new type that acts as an `usize`-based replacement for the old
type that can be used as a key for `blazemap` collections.

Being an analogue of `define_key_wrapper!`
for the case when the user could statically guarantee
that the number of unique keys doesn't exceed `MAX_CAP`, it's optimized for
read operations so that they don't create any multi-thread contention.

#### Example

```rust
use blazemap::prelude::{BlazeMap, define_key_wrapper_bounded};

define_key_wrapper_bounded! {
    pub struct Key(&'static str);
    MAX_CAP = 40_000;
    Derive(as for Original Type): {  // Optional section
        Debug,
        Display,
    };
    Derive(as for usize): {          // Optional section
        Ord,
    }
}

let key_1 = Key::new("first");
let key_2 = Key::new("second");
let key_3 = Key::new("third");

let mut map = BlazeMap::new();
map.insert(key_2, "2");
map.insert(key_1, "1");
map.insert(key_3, "3");

assert_eq!(format!("{map:?}"), r#"{"first": "1", "second": "2", "third": "3"}"#)
```

### 3. `define_plain_id!`

Creates a new type based on incrementally generated `usize` instances
that can be used as a key for `blazemap` collections. This is the most performant way to generate keys for `BlazeMap`.

#### Example
```rust
use blazemap::prelude::{BlazeMap, define_plain_id};

define_plain_id! {
    pub struct Id;
    Derive: {       // Optional section
        Ord
    };
}

let key_1 = Id::new();
let key_2 = Id::new();
let key_3 = Id::new();

let mut map = BlazeMap::new();
map.insert(key_2, "2");
map.insert(key_1, "1");
map.insert(key_3, "3");

assert_eq!(format!("{map:?}"), r#"{0: "1", 1: "2", 2: "3"}"#)
```