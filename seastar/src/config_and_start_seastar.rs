use cxx::UniquePtr;
use ffi::*;

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
        // These run the app
        fn run_void(app: &UniquePtr<app_template>, args: &Vec<String>, func: fn()) -> i32;
        fn run_int(app: &UniquePtr<app_template>, args: &Vec<String>, func: fn() -> i32) -> i32;
    }
}

/// The configuration of an [`AppTemplate`] instance.
/// Some of the options are just metadata, others affect the app's performance.
pub struct Options {
    opts: UniquePtr<seastar_options>,
}

impl Options {
    /// Creates a default instance of `Options`.
    ///
    /// # Seastar's defaults
    ///
    /// - `name` - "App",
    /// - `description` - "" (empty),
    /// - `smp` - number of threads/cores (equal to [`num_cpus::get()`]).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use seastar::{AppTemplate, Options};
    ///
    /// let opts = Options::new();
    /// let app = AppTemplate::new_from_options(opts);
    /// ```
    pub fn new() -> Self {
        Options {
            opts: new_options(),
        }
    }

    /// Gets the `Options`' name.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use seastar::{AppTemplate, Options};
    ///
    /// let opts = Options::new();
    ///
    /// assert_eq!(opts.get_name(), "App");
    /// ```
    pub fn get_name(&self) -> &str {
        get_name(&self.opts)
    }

    /// Gets the `Options`' description.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use seastar::{AppTemplate, Options};
    ///
    /// let opts = Options::new();
    ///
    /// assert_eq!(opts.get_description(), "");
    /// ```
    pub fn get_description(&self) -> &str {
        get_description(&self.opts)
    }

    /// Gets the `Options`' number of threads.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use seastar::{AppTemplate, Options};
    ///
    /// let opts = Options::new();
    ///
    /// assert_eq!(opts.get_smp(), num_cpus::get() as u32);
    /// ```
    pub fn get_smp(&self) -> u32 {
        get_smp(&self.opts)
    }

    /// Sets the `Options`' name.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use seastar::{AppTemplate, Options};
    ///
    /// let mut opts = Options::new();
    /// let name = "My Awesome Seastar App";
    /// opts.set_name(name);
    ///
    /// assert_eq!(opts.get_name(), name);
    /// ```
    pub fn set_name(&mut self, name: &str) {
        set_name(&self.opts, name);
    }

    /// Sets the `Options`' description.
    ///
    /// # Examples
    ///
    ///```rust
    /// use seastar::{AppTemplate, Options};
    ///
    /// let mut opts = Options::new();
    /// let description = "I love Seastar";
    /// opts.set_description(description);
    ///
    /// assert_eq!(opts.get_description(), description);
    /// ```
    pub fn set_description(&mut self, description: &str) {
        set_description(&self.opts, description);
    }

    /// Sets the `Options`' number of threads.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use seastar::{AppTemplate, Options};
    ///
    /// let mut opts = Options::new();
    /// let smp = 42;
    /// opts.set_smp(smp);
    ///
    /// assert_eq!(opts.get_smp(), smp);
    /// ```
    pub fn set_smp(&mut self, smp: u32) {
        set_smp(&self.opts, smp);
    }
}

impl Default for Options {
    fn default() -> Self {
        Options::new()
    }
}

/// The object through which the contents of a `main` function would be ran in a Seastar app.
/// Configurable through [`Options`].
pub struct AppTemplate {
    app: UniquePtr<app_template>,
}

impl AppTemplate {
    /// Creates a Seastar app based on its configuration (`[seastar-rs::AppTemplate`]).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use seastar::{AppTemplate, Options};
    ///
    /// let app = AppTemplate::new_from_options(Options::default());
    /// ```
    pub fn new_from_options(opts: Options) -> Self {
        AppTemplate {
            app: new_app_template_from_options(&opts.opts),
        }
    }

    /// Runs an app with a void callback (the output of which is always 0) and program arguments (argv).
    /// The app is run on a separate thread.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use seastar::{AppTemplate, Options};
    ///
    /// let func = || { println!("{}", 42); };
    ///
    /// let app = AppTemplate::default();
    /// let args = vec!["hello".to_owned()];
    ///
    /// assert_eq!(app.run_void(&args, func), 0);
    /// ```
    pub fn run_void(&self, args: &Vec<String>, func: fn()) -> i32 {
        run_void(&self.app, args, func)
    }

    /// Runs an app with an int (status code) callback and program arguments (argv).
    /// The app is run on a separate thread.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use seastar::{AppTemplate, Options};
    ///
    /// let func = || { 42 };
    ///
    /// let app = AppTemplate::default();
    /// let args = vec!["hello".to_owned()];
    ///
    /// assert_eq!(app.run_int(&args, func), 42);
    /// ```
    pub fn run_int(&self, args: &Vec<String>, func: fn() -> i32) -> i32 {
        run_int(&self.app, args, func)
    }
}

impl Default for AppTemplate {
    fn default() -> Self {
        AppTemplate::new_from_options(Options::default())
    }
}
/*
RUSTFLAGS="-C link-arg=-fuse-ld=lld" \
RUSTDOCFLAGS="-C link-arg=-fuse-ld=lld" \
cargo test
*/
#[cfg(test)]
mod tests {
    use super::*;

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
        let app = AppTemplate::default();
        let args = vec![String::from("test")];
        fn func() -> i32 {
            42
        }
        assert_eq!(app.run_int(&args, func), 42);
    }

    #[test]
    fn test_run_void() {
        let app = AppTemplate::default();
        let args = vec![String::from("test")];
        fn func() {}
        assert_eq!(app.run_void(&args, func), 0);
    }
}
