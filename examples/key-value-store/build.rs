fn main() {
    let seastar = pkg_config::Config::new()
        .statik(true)
        .probe("seastar")
        .unwrap();

    pkg_config::Config::new().statik(true).probe("fmt").unwrap();

    let mut build = cxx_build::bridge("src/net_ffi.rs");
    for (var, value) in &seastar.defines {
        match value {
            Some(val) => build.define(var, val.as_str()),
            None => build.define(var, None),
        };
    }
    build
        .flag_if_supported("-Wall")
        .flag_if_supported("-std=c++20")
        .flag_if_supported("-fcoroutines")
        .includes(&seastar.include_paths)
        .cpp_link_stdlib("stdc++")
        .file("src/net_ffi.cc")
        .compile("key-value-store");
}
