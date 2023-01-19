/// Convert the pointer to Box<Func> and then call it, consuming it in the process.
fn fn_once_caller<Func, Ret>(raw_func: *mut u8) -> Ret
where
    Func: FnOnce() -> Ret,
{
    unsafe { Box::from_raw(raw_func as *mut Func)() }
}

/// A helper function. We need to be able to name the type of a named closure.
/// There is no `typeof` in Rust, so this is achieved by using a dummy parameter.
pub const fn get_fn_once_caller<Func, Ret>(_: &Func) -> fn(*mut u8) -> Ret
where
    Func: FnOnce() -> Ret,
{
    fn_once_caller::<Func, Ret>
}

/// Free a pointer.
fn dropper<T>(raw_ptr: *mut u8) {
    unsafe {
        let _ = Box::from_raw(raw_ptr as *mut T);
    }
}

pub const fn get_dropper<T>(_: &T) -> fn(*mut u8) {
    dropper::<T>
}
