#[cxx::bridge(namespace = "net_ffi")]
mod ffi {
    unsafe extern "C++" {
        include!("key-value-store/src/net_ffi.hh");
    }
}
