# seastar-rs

Idiomatic bindings to the Seastar framework.

__This crate is very much a work in progress!__ Do not use in production (yet).

## How to build

seastar-rs finds the compiled seastar library and its dependencies through `pkg-config`.
This means that seastar and each dependency either needs to be installed in the system, or it must be pointed to through the `PKG_CONFIG_PATH` environment variable.

For developers that need to modify the seastar library frequently, the most useful workflow would be to install all dependencies through the [`./install-dependencies.sh`](https://github.com/scylladb/seastar/blob/master/install-dependencies.sh) script but specify seastar through `pkg-config`:

```bash
export PKG_CONFIG_PATH="/your/path/to/seastar/build/release/:$PKG_CONFIG_PATH"
cargo build
```

__Important__: seastar-rs currently requires seastar to be built with the `-fpie` flag. Usage of clang is also heavily recommended:

```bash
# In seastar directory
./configure.py --cflags="-fpie" --compiler=clang++ --c-compiler=clang --mode=release --without-demos --without-tests --without-apps
ninja -C build/release
```

## Coding style

See [coding-style.md](./coding-style.md).
