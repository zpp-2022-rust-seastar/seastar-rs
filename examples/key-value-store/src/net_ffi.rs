use cxx::UniquePtr;
use std::future::Future;

#[cxx::bridge(namespace = "net_ffi")]
mod ffi {
    unsafe extern "C++" {
        include!("key-value-store/src/net_ffi.hh");

        type VoidFuture = crate::VoidFuture;
        type StringFuture = crate::StringFuture;

        #[rust_name = "ServerSocket"]
        type server_socket;

        #[rust_name = "ConnectedSocket"]
        type connected_socket;

        #[rust_name = "InputStream"]
        type input_stream;

        #[rust_name = "OutputStream"]
        type output_stream;

        fn listen(port: u16) -> UniquePtr<ServerSocket>;

        fn accept(
            server_socket: &UniquePtr<ServerSocket>,
            socket: &mut UniquePtr<ConnectedSocket>,
        ) -> VoidFuture;

        fn get_input_stream(socket: &UniquePtr<ConnectedSocket>) -> UniquePtr<InputStream>;

        fn get_output_stream(socket: &UniquePtr<ConnectedSocket>) -> UniquePtr<OutputStream>;

        fn close_output_stream(socket: &UniquePtr<OutputStream>) -> VoidFuture;

        fn read(socket: &UniquePtr<InputStream>) -> StringFuture;

        fn write(socket: &UniquePtr<OutputStream>, msg: &str) -> VoidFuture;

        fn flush_output(socket: &UniquePtr<OutputStream>) -> VoidFuture;
    }
}

#[cxx_async::bridge(namespace = net_ffi)]
unsafe impl Future for VoidFuture {
    type Output = ();
}

#[cxx_async::bridge(namespace = net_ffi)]
unsafe impl Future for StringFuture {
    type Output = String;
}

pub use ffi::{
    close_output_stream, flush_output, get_input_stream, get_output_stream, listen, read, write,
    ConnectedSocket, InputStream, OutputStream, ServerSocket,
};

pub async fn accept(
    server_socket: &UniquePtr<ServerSocket>,
) -> Result<UniquePtr<ConnectedSocket>, cxx_async::CxxAsyncException> {
    let mut socket = UniquePtr::null();
    ffi::accept(server_socket, &mut socket).await?;
    Ok(socket)
}
