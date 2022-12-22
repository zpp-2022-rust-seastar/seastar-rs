use std::path::PathBuf;

static CXX_BRIDGES: &[&str] = &[
    // Put all files that contain a cxx::bridge into this list
    "src/preempt.rs",
];

fn main() {
    let seastar = pkg_config::Config::new()
        .statik(true)
        .probe("seastar")
        .unwrap();

    // Workaround for the fact that seastar's pkg-config file
    // specifies the fmt dependency in a weird way. `pkg-config seastar --libs`
    // prints a path to a particular version of fmt (e.g. libfmt.so.8.1.1)
    // and the pkg_config crate can't parse this name as it expects to end
    // with just ".so". pkg_config crate prints a warning and does not
    // tell cargo to link with that library, so we have to do it manually.
    // Unfortunately, this workaround doesn't prevent a warning from being
    // printed by the previous command which prevents us from enforcing
    // a no-warning policy in the CI.
    // TODO: Remove this after seastar.pc or the pkg-config crate is fixed
    pkg_config::Config::new().statik(true).probe("fmt").unwrap();

    // TODO: liburing probably has the same problem as above

    let cxx_bridges = CXX_BRIDGES
        .iter()
        .map(|p| PathBuf::try_from(p).unwrap())
        .collect::<Vec<_>>();

    // TODO: The API level and scheduling group count
    // should be configurable somehow
    cxx_build::bridges(&cxx_bridges)
        .flag_if_supported("-Wall")
        .flag_if_supported("-std=c++20")
        .flag_if_supported("-fcoroutines")
        .define("SEASTAR_API_LEVEL", "6")
        .define("SEASTAR_SCHEDULING_GROUPS_COUNT", "16")
        .includes(&seastar.include_paths)
        .compile("seastar-rs");

    println!("cargo:rerun-if-changed=build.rs");
    for bridge_file in cxx_bridges.iter() {
        println!("cargo:rerun-if-changed={}", bridge_file.to_str().unwrap());
    }
}
