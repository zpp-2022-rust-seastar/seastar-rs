#[cxx::bridge]
mod ffi {
    #[namespace = "seastar_ffi::clocks"]
    unsafe extern "C++" {
        include!("seastar/src/clocks.hh");
    }
}

use ffi::*;
