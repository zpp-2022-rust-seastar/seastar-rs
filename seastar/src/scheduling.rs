#[cxx::bridge(namespace = "seastar_ffi::scheduling")]
mod ffi {
    unsafe extern "C++" {
        include!("seastar/src/scheduling.hh");
    }
}

use ffi::*;
