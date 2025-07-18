# dewey-decimal

[![Latest Version](https://img.shields.io/crates/v/dewey-decimal.svg)](https://crates.io/crates/dewey-decimal)
[![docs.rs](https://img.shields.io/docsrs/dewey-decimal)](https://docs.rs/dewey-decimal)
![GitHub License](https://img.shields.io/github/license/dax-dot-gay/dewey-decimal)

Simple wrapper around Dewey Decimal classifications

### Usage

```rust
// Complete documentation: https://docs.rs/dewey-decimal

use dewey_decimal::{Dewey, Class};

fn main() {
    // Get the class representing "Computer science, knowledge & systems"
    let comp_sci = Class::get("00").unwrap();

    // Gets all children in this class
    let cs_classes = comp_sci.all_children()
}
```
