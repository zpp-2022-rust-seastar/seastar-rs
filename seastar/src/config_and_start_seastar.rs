use std::future::Future;

use cxx::UniquePtr;
use ffi::*;

use crate::cxx_async_local_future::IntoCxxAsyncLocalFuture;

#[cxx::bridge(namespace = "seastar_ffi::config_and_start_seastar")]
mod ffi {
    unsafe extern "C++" {
        include!("seastar/src/config_and_start_seastar.hh");

        type seastar_options;
        type app_template;

        type VoidFuture = crate::cxx_async_futures::VoidFuture;
        type IntFuture = crate::cxx_async_futures::IntFuture;

        // Returns a pointer to default `seastar_options`
        fn new_options() -> UniquePtr<seastar_options>;
        // Getters
        fn get_name(opts: &seastar_options) -> &str;
        fn get_description(opts: &seastar_options) -> &str;
        fn get_smp(opts: &seastar_options) -> u32;
        // Setters
        fn set_name(opts: Pin<&mut seastar_options>, name: &str);
        fn set_description(opts: Pin<&mut seastar_options>, description: &str);
        fn set_smp(opts: Pin<&mut seastar_options>, smp: u32);

        // Returns a pointer to an `app_template` instance
        fn new_app_template_from_options(
            opts: Pin<&mut seastar_options>,
        ) -> UniquePtr<app_template>;
        // These run the app
        fn run_void(app: Pin<&mut app_template>, args: &[&str], fut: VoidFuture) -> i32;
        fn run_int(app: Pin<&mut app_template>, args: &[&str], fut: IntFuture) -> i32;
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
        set_name(self.opts.pin_mut(), name);
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
        set_description(self.opts.pin_mut(), description);
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
        set_smp(self.opts.pin_mut(), smp);
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
    pub fn new_from_options(mut opts: Options) -> Self {
        AppTemplate {
            app: new_app_template_from_options(opts.opts.pin_mut()),
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
    /// let fut = async move { Ok(()) };
    ///
    /// let mut app = AppTemplate::default();
    /// let args = vec!["hello"];
    ///
    /// assert_eq!(app.run_void(&args[..], fut), 0);
    /// ```
    pub fn run_void(
        &mut self,
        args: &[&str],
        fut: impl Future<Output = cxx_async::CxxAsyncResult<()>> + 'static,
    ) -> i32 {
        run_void(self.app.pin_mut(), args, VoidFuture::fallible_local(fut))
    }

    /// Runs an app with an int (status code) callback and program arguments (argv).
    /// The app is run on a separate thread.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use seastar::{AppTemplate, Options};
    ///
    /// let fut = async move { Ok(42) };
    ///
    /// let mut app = AppTemplate::default();
    /// let args = vec!["hello"];
    ///
    /// assert_eq!(app.run_int(&args[..], fut), 42);
    /// ```
    pub fn run_int(
        &mut self,
        args: &[&str],
        fut: impl Future<Output = cxx_async::CxxAsyncResult<i32>> + 'static,
    ) -> i32 {
        run_int(self.app.pin_mut(), args, IntFuture::fallible_local(fut))
    }
}

impl Default for AppTemplate {
    fn default() -> Self {
        AppTemplate::new_from_options(Options::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

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
        thread::spawn(|| {
            let _guard = crate::acquire_guard_for_seastar_test();
            let mut app = AppTemplate::default();
            let args = vec!["test"];
            let fut = async { Ok(42) };
            assert_eq!(app.run_int(&args[..], fut), 42);
        })
        .join()
        .unwrap();
    }

    #[test]
    fn test_run_void() {
        thread::spawn(|| {
            let _guard = crate::acquire_guard_for_seastar_test();
            let mut app = AppTemplate::default();
            let args = vec!["test"];
            let fut = async { Ok(()) };
            assert_eq!(app.run_void(&args[..], fut), 0);
        })
        .join()
        .unwrap();
    }
}
