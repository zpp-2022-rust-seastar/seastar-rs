pub use crate::config_and_start_seastar::ffi::*;
use cxx::{CxxString, UniquePtr};

#[cxx::bridge(namespace = "seastar")]
mod ffi {

    unsafe extern "C++" {
        include!("seastar/src/config_and_start_seastar.hh");
        type seastar_options;
        type app_template;

        // Returns a pointer to default `seastar_options`
        fn new_options() -> UniquePtr<seastar_options>;
        // Getters
        fn get_name(opts: &UniquePtr<seastar_options>) -> &str;
        fn get_description(opts: &UniquePtr<seastar_options>) -> &str;
        fn get_smp(opts: &UniquePtr<seastar_options>) -> u32;
        // Setters
        fn set_name(opts: &UniquePtr<seastar_options>, name: &str);
        fn set_description(opts: &UniquePtr<seastar_options>, description: &str);
        fn set_smp(opts: &UniquePtr<seastar_options>, smp: u32);

        // Returns a pointer to an `app_template` instance
        fn new_app_template_from_options(
            opts: &UniquePtr<seastar_options>,
        ) -> UniquePtr<app_template>;

        // fn run_void(app: &UniquePtr<app_template>, func: fn()) -> i32;
        // fn run_int(app: &UniquePtr<app_template>, func: fn() -> i32) -> i32;
    }
}

struct Options {
    opts: UniquePtr<seastar_options>,
}

impl Options {
    fn new() -> Self {
        Options {
            opts: new_options(),
        }
    }

    fn get_name(&self) -> &str {
        get_name(&self.opts)
    }

    fn get_description(&self) -> &str {
        get_description(&self.opts)
    }

    // Gets the number of threads (default: one per CPU)
    fn get_smp(&self) -> u32 {
        get_smp(&self.opts)
    }

    fn set_name(&mut self, name: &str) {
        set_name(&mut self.opts, name);
    }

    fn set_description(&mut self, description: &str) {
        set_description(&mut self.opts, description);
    }

    // Sets the number of threads (default: one per CPU)
    fn set_smp(&mut self, smp: u32) {
        set_smp(&mut self.opts, smp);
    }
}

impl Default for Options {
    fn default() -> Self {
        Options::new()
    }
}

struct AppTemplate {
    app: UniquePtr<app_template>,
}

impl AppTemplate {
    fn new_from_options(opts: Options) -> Self {
        AppTemplate {
            app: new_app_template_from_options(&opts.opts),
        }
    }

    fn run_void(&self, func: fn()) -> i32 {
        todo!()
    }

    fn run_int(&self, func: fn() -> i32) -> i32 {
        todo!()
    }
}

#[test]
fn test_new_options_contain_default_values() {
    let opts = Options::new();
    assert_eq!(opts.get_name(), "App");
    assert_eq!(opts.get_description(), "");
    assert_eq!(opts.get_smp(), num_cpus::get() as u32);
}

#[test]
fn test_set_get_name() {
    let mut opts = Options::new();
    let name = "42";
    opts.set_name(name);
    assert_eq!(opts.get_name(), name);
}

#[test]
fn test_set_get_description() {
    let mut opts = Options::new();
    let description = "42";
    opts.set_description(description);
    assert_eq!(opts.get_description(), description);
}

#[test]
fn test_set_get_smp() {
    let mut opts = Options::new();
    let smp = 42;
    opts.set_smp(smp);
    assert_eq!(opts.get_smp(), smp);
}

#[test]
fn test_new_app_template_from_options_gets_created() {
    let mut opts = Options::default();
    opts.set_name("42");
    opts.set_description("42");
    opts.set_smp(42);
    let _ = AppTemplate::new_from_options(opts);
}

#[test]
fn test_run_int() {
    //TODO
}

#[test]
fn test_run_void() {
    //TODO
}
