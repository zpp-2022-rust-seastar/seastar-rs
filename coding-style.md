# Coding style

For C++ code, the [seastar coding style](https://github.com/scylladb/seastar/blob/master/coding-style.md) should be used.

The rest of the document concerns the Rust code.

## Formatting

Use `cargo fmt` to format the code.

## General style

Use `cargo clippy` to apply generic lints.

## Naming

Items exported from seastar should have the same name as the original item.
Exceptions are allowed in reasonable and justified cases.

One important exception is usage of CamelCase vs. snake_case. If Rust uses a different convention that seastar for a particular type of item, then the name should be converted to the appropriate convention. Most notably, seastar uses snake case for type names while in Rust, camel case is used. For example, `seastar::app_template` should be re-exported in Rust as `seastar::AppTemplate`.

## Code organization

When exporting an item from seastar, the module path should correspond to the namespace in the original C++ code.
That said, it is fine to first define the Rust item in a different module path and then re-export it in the correct one.
In seastar, most items are defined just in the `seastar` namespace - in C++ it's possible for multiple files to add items to the same namespace.
In Rust, it is impossible to split a single module across multiple files, and we don't want to put everything into a single file.
Therefore, the next best thing is to define a Rust item in a submodule and then re-export it in a parent module.

For example:

```rust
// preempt.rs
// Corresponds to seastar::need_preempt().
// Needs to be re-exported in the crate root
extern "C" fn need_preempt() -> bool;
```

```rust
// lib.rs
mod preempt; // Can be a private module

// The name of the crate is `seastar`, so just re-export the name
// in the crate root. You can use wildcards.
pub use preempt::*;
```

## Documenting code

Ideally, all items should be documented. Many items that need to be exported from the C++ seastar code already have docstrings, so usually it will be just a matter of copying the docstring and adjusting it to the conventions used in Rust.

## Other

When in doubt, refer to the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/about.html) document.
