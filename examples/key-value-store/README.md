# key-value store

This crate is an example of using the crate `seastar-rs`. Its two main objectives are:
1. To demonstrate how to use some of Seastar's functionalities exposed by `seastar-rs` from C++ to Rust. This example covers [`seastar::main`](https://github.com/zpp-2022-rust-seastar/seastar-rs/blob/main/seastar-macros/src/lib.rs) macro and [`seastar::Distributed<S>`](https://github.com/zpp-2022-rust-seastar/seastar-rs/blob/main/seastar/src/distributed.rs).
2. To show how to integrate the code written using `seastar-rs `with the already existing C++ code. In this example, a part of Seastar's network API is exposed to Rust using the `cxx` crate and integrated with the rest of the Rust code.

## Use

To run this example, execute the following command in the `seastar-rs` directory:
```
RUSTFLAGS="-C link-arg=-fuse-ld=lld" RUSTDOCFLAGS="-C link-arg=-fuse-ld=lld" cargo run --bin key-value-store
```

If you work on a machine with more than 30 cores, running the example can end up in a deadlock due to a bug in the `cxx-async` crate. In such a case, the `taskset` command might help. For instance, you could run the example using 16 cores like this:

```
RUSTFLAGS="-C link-arg=-fuse-ld=lld" RUSTDOCFLAGS="-C link-arg=-fuse-ld=lld" taskset -c 0-15 cargo run --bin key-value-store
```

If you want to connect to the server, you could use `netcat`:
```
nc <address> 5555
```

## Server

This example is an implementation of a simple key-value store server. The server accepts TCP connections on port 5555 and serves client requests. Each request is an ASCII string.

### Requests

The server accepts two kinds of requests - STORE and LOAD.

STORE requests:
  - have a form `STORE$key$value\n` where `key` and `value` can be a string of any length (even empty) containing only lowercase letters of the English alphabet,
  - after receiving such a request, the server puts `value` under `key` in its memoty,
  - if `key` is already present in the server's memory, `value` overwrites the old value,
  - the server responds with `DONE\n`.

LOAD requests:
  - have a form `LOAD$key\n` where `key` can be a string of any length containing only lowercase letters of the English alphabet,
  - after receiving such a request, if `key` is present in the server's memory, the server sends a reposone of a form `FOUND$value\n` where `value` is the value under `key`,
  - if `key` is not present in the server's memory, the server sends a response `NOTFOUND\n`.

If the server receives an incorrect request, it closes the connection with the client. More precisely, this happens when a message sent by the client cannot become a correct STORE or LOAD request, no matter what the client sends in the future.

### Internals

Understanding the server's logic might be helpful when reading the code. In short, the server works like this:
- The server is distributed on all shards. To achieve this, it uses `seastar::Distributed<S>`.
- Every shard owns a key-value store called `db` responsible for a part of the keys. The `db` on shard `s` stores key `k` if and only if `hash(k) % num_shards = s` where `hash` is a hashing function.
- All shards are listening concurrently on port 5555.
- After accepting a new client, the server spawns an independent `handle_connection` task on the same shard using `seastar::Distributed<S>::map_current`. This task is not `await`ed since we do not want to stop a listening loop for a single client. The task continues until an error occurs, the client disconnects, or the client sends an incorrect message.
- To perform a STORE or LOAD request, the server spawns a new task on a shard that owns `db` responsible for this request's key. Communication between shards is possible thanks to `seastar::PeeringShardedService<'a, S>`.
- When the server processes a message sent by a client, 3 cases are possible:
  - The message contains a prefix (perhaps equal to the whole message) that is a proper request. In this case, the server handles the request in the way described above and continues processing the message after removing the request from it.
  - The message does not contain a prefix being a complete request, but such a prefix may appear when the client sends new characters. Examples of such a message could be `STORE$key$va` and `LOA`. In this case, the server waits for future characters.
  - The message does not contain a prefix being a complete request, and it is impossible that such a prefix will appear in the future no matter what the client sends. Examples of such a message could be `STORE$key$@`, `LOAD$Key` and `STTORE`. In this case, the server closes the connection with the client.
