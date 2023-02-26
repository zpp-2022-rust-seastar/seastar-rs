#[cxx::bridge(namespace = "seastar_ffi::gate")]
mod ffi {
    unsafe extern "C++" {
        include!("seastar/src/gate.hh");
    }
}

use ffi::*;
