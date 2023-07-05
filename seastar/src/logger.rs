use std::fmt::Arguments;
use std::pin::Pin;

use cxx::UniquePtr;

#[cxx::bridge(namespace = "seastar_ffi::logger")]
mod ffi {
    extern "Rust" {
        type FormatCtx<'a>;
        fn write_log_line(writer: Pin<&mut log_writer>, ctx: &FormatCtx<'_>);
    }

    unsafe extern "C++" {
        include!("seastar/src/logger.hh");

        type log_writer;
        fn write(self: Pin<&mut log_writer>, data: &[u8]);

        type logger;
        fn new_logger(name: &str) -> UniquePtr<logger>;
        fn log(l: &logger, level: u32, ctx: &FormatCtx<'_>);
    }
}

/// Internal, do not use.
// For some reason, cxx requires this to be public.
#[doc(hidden)]
pub struct FormatCtx<'a> {
    args: Arguments<'a>,
}

fn write_log_line(writer: Pin<&mut ffi::log_writer>, ctx: &FormatCtx<'_>) {
    struct FmtWriter<'a>(Pin<&'a mut ffi::log_writer>);
    impl<'a> std::fmt::Write for FmtWriter<'a> {
        fn write_str(&mut self, s: &str) -> std::fmt::Result {
            self.0.as_mut().write(s.as_bytes());
            Ok(())
        }
    }

    std::fmt::write(&mut FmtWriter(writer), ctx.args).unwrap();
}

/// Log verbosity level.
#[repr(u32)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

/// A wrapper over seastar::logger.
///
/// # Usage
///
/// Customarily, seastar loggers are created on a per-module basis. Rust doesn't
/// support non-const constructors for static globals, but you can use
/// the [`ctor`] crate to achieve this:
///
/// ```rust
/// use ctor::ctor;
/// use seastar::Logger;
///
/// #[ctor]
/// static LOGGER: Logger = Logger::new("my_logger");
/// ```
///
/// Printing messages is done with the help of Rust's formatting infrastructure.
/// There are five main macros, one for each log level:
/// [`error!`](crate::error!), [`warn!`](crate::warn!), [`info!`](crate::info!),
/// [`debug!`](crate::debug!) and [`trace!`](crate::trace!). They take
/// a logger as a first argument and then the rest of the arguments is the same
/// as for e.g. the [`println!`](std::println!) macro.
///
/// ```rust
/// # use seastar::Logger;
/// # fn compile_only() {
/// let logger = Logger::new("my_logger");
/// seastar::trace!(logger, "Some verbose stuff");
/// seastar::info!(logger, "The answer is: {}", 42);
/// # }
/// ```
pub struct Logger {
    core: UniquePtr<ffi::logger>,
}

unsafe impl Send for Logger {}
unsafe impl Sync for Logger {}

impl Logger {
    /// Creates a new seastar logger with given name.
    ///
    /// # Example
    ///
    /// In C++, seastar loggers are usually declared as static global variables.
    /// They register themselves in a global registry before the main function
    /// runs, allowing the application to obtain information about available
    /// loggers at any point, set the verbosity level etc.
    ///
    /// Rust doesn't support non-const constructors for static variables
    /// out of the box, but they can be introduced with the `ctor` crate.
    ///
    /// ```rust
    /// # use ctor::ctor;
    /// # use seastar::Logger;
    /// #[ctor]
    /// static LOGGER: Logger = Logger::new("my_logger");
    ///
    /// async fn do_stuff() {
    ///     seastar::info!(LOGGER, "Doing stuff...");
    /// }
    /// ```
    #[inline]
    pub fn new(name: &str) -> Self {
        Self {
            core: ffi::new_logger(name),
        }
    }

    /// Emits a message with requested level.
    ///
    /// While it's possible to use directly, you will most likely be
    /// interested in the [`log!`](crate::log!) macro instead.
    #[inline]
    pub fn log(&self, level: LogLevel, args: Arguments<'_>) {
        let ctx = FormatCtx { args };
        ffi::log(&self.core, level as u32, &ctx);
    }

    /// Emits a `trace` level message.
    ///
    /// While it's possible to use directly, you will most likely be
    /// interested in the [`trace!`](crate::trace!) macro instead.
    #[inline]
    pub fn trace(&self, args: Arguments<'_>) {
        self.log(LogLevel::Trace, args);
    }

    /// Emits a `debug` level message.
    ///
    /// While it's possible to use directly, you will most likely be
    /// interested in the [`debug!`](crate::debug!) macro instead.
    #[inline]
    pub fn debug(&self, args: Arguments<'_>) {
        self.log(LogLevel::Debug, args);
    }

    /// Emits a `info` level message.
    ///
    /// While it's possible to use directly, you will most likely be
    /// interested in the [`info!`](crate::info!) macro instead.
    #[inline]
    pub fn info(&self, args: Arguments<'_>) {
        self.log(LogLevel::Info, args);
    }

    /// Emits a `warn` level message.
    ///
    /// While it's possible to use directly, you will most likely be
    /// interested in the [`warn!`](crate::warn!) macro instead.
    #[inline]
    pub fn warn(&self, args: Arguments<'_>) {
        self.log(LogLevel::Warn, args);
    }

    /// Emits a `error` level message.
    ///
    /// While it's possible to use directly, you will most likely be
    /// interested in the [`error!`](crate::error!) macro instead.
    #[inline]
    pub fn error(&self, args: Arguments<'_>) {
        self.log(LogLevel::Error, args);
    }
}

/// Emits a formatted log message with given logger.
///
/// The arguments to the macro are as follows:
/// - `logger` - reference to the [`Logger`] to be used,
/// - `level` - [`LogLevel`] to use,
/// - `arg...` - arguments, as if passed to the [`std::format!`] macro.
///
/// # Example
/// ```rust
/// # use seastar::{Logger, LogLevel};
/// # fn compile_only() {
/// let logger = Logger::new("my_logger");
/// seastar::log!(logger, LogLevel::Info, "Hello, {}!", "world");
/// # }
/// ```
#[macro_export]
macro_rules! log {
    ($logger:expr, $level:expr, $($arg:tt),*) => {{
        $logger.log($level, std::format_args!($($arg),*))
    }};
}

/// Emits a formatted log message with `trace` level.
///
/// Equivalent to calling [`log!`](crate::log!) with `trace` level.
#[macro_export]
macro_rules! trace {
    ($logger:expr, $($arg:tt),*) => {{
        $logger.trace(std::format_args!($($arg),*))
    }};
}

/// Emits a `debug` level message with given logger.
///
/// Equivalent to calling [`log!`](crate::log!) with `debug` level.
#[macro_export]
macro_rules! debug {
    ($logger:expr, $($arg:tt),*) => {{
        $logger.debug(std::format_args!($($arg),*))
    }};
}

/// Emits a `info` level message with given logger.
///
/// Equivalent to calling [`log!`](crate::log!) with `info` level.
#[macro_export]
macro_rules! info {
    ($logger:expr, $($arg:tt),*) => {{
        $logger.info(std::format_args!($($arg),*))
    }};
}

/// Emits a `warn` level message with given logger.
///
/// Equivalent to calling [`log!`](crate::log!) with `warn` level.
#[macro_export]
macro_rules! warn {
    ($logger:expr, $($arg:tt),*) => {{
        $logger.warn(std::format_args!($($arg),*))
    }};
}

/// Emits an `error` level message with given logger.
///
/// Equivalent to calling [`log!`](crate::log!) with `error` level.
#[macro_export]
macro_rules! error {
    ($logger:expr, $($arg:tt),*) => {{
        $logger.error(std::format_args!($($arg),*))
    }};
}
