# tralllocator
[![crates.io](https://img.shields.io/crates/v/trallocator)](https://crates.io/crates/trallocator) [![documentation](https://docs.rs/trallocator/badge.svg)](https://docs.rs/trallocator)


A simple `no_std` library for wrapping an existing allocator and tracking the heap usage.
Main usage is to keep track of the heap usage on embedded systems

## Usage 
Simply wrap an existing allocator with a `Trallocator`.

### Examples
With another allocator, here we use the system one
``` rust
# extern crate alloc;
# extern crate std;
use alloc::vec::Vec;
use std::alloc::System;

use trallocator::Trallocator;

#[global_allocator]
static ALLOCATOR: Trallocator<System> = Trallocator::new(System);
fn main() {
    let init = ALLOCATOR.usage();
    let mut vec: Vec<u8> = Vec::new();
    vec.reserve_exact(32);
    assert_eq!(ALLOCATOR.usage(), init+32);
}
```

With the [allocator API](https://github.com/rust-lang/rust/issues/32838)
``` rust
#![feature(allocator_api)]
# extern crate alloc;
# extern crate std;
use alloc::vec::Vec;
use std::alloc::System;

use trallocator::Trallocator;

let tralloc: Trallocator<System> = Trallocator::new(System);
assert_eq!(tralloc.usage(), 0);
let mut vec: Vec<u8, _> = Vec::new_in(&tralloc);
vec.reserve_exact(32);
assert_eq!(tralloc.usage(), 32);
```

### License
Licensed under either of
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
