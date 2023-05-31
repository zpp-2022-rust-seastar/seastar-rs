use super::*;
use cxx::UniquePtr;
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

// An error returned when something fails while handling requests.
// We do not care what really happened because in every case we just
// close the connection with the client.
#[derive(Debug)]
pub struct ConnectionError;

pub type ConnectionResult<T> = Result<T, ConnectionError>;

pub struct Connection {
    _socket: UniquePtr<ConnectedSocket>,
    input: UniquePtr<InputStream>,
    output: UniquePtr<OutputStream>,
    message: String, // Unprocessed fragment of the message.
}

impl Connection {
    fn new(
        _socket: UniquePtr<ConnectedSocket>,
        input: UniquePtr<InputStream>,
        output: UniquePtr<OutputStream>,
    ) -> Self {
        Self {
            _socket,
            input,
            output,
            message: String::new(),
        }
    }
}

pub struct Server {
    db: RefCell<HashMap<String, String>>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            db: RefCell::new(HashMap::new()),
        }
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl seastar::Service for Server {}

pub struct ShardedServer<'a>(pub seastar::PeeringShardedService<'a, Server>);

impl ShardedServer<'_> {
    #[allow(unused_must_use)]
    pub async fn run(self, port: u16) {
        let listener = listen(port);

        loop {
            if let Ok(socket) = accept(&listener).await {
                self.0.container.map_current(|sharded| async move {
                    let input = get_input_stream(&socket);
                    let output = get_output_stream(&socket);
                    Self(sharded)
                        .handle_connection(Connection::new(socket, input, output))
                        .await;
                });
            }
        }
    }

    async fn handle_connection(self, mut conn: Connection) {
        loop {
            match read(&conn.input).await {
                Err(_) => break,
                Ok(buffer) if buffer.is_empty() => break,
                Ok(buffer) => {
                    buffer.chars().for_each(|c| conn.message.push(c));

                    if self.process_message(&mut conn).await.is_err() {
                        break;
                    }
                }
            }
        }

        let _ = close_output_stream(&conn.output).await;
    }

    // Processes message until it has no prefix being a complete STORE or LOAD request.
    // Returns ConnectionError, if message is for sure incorrect.
    async fn process_message(&self, conn: &mut Connection) -> ConnectionResult<()> {
        loop {
            match try_parse_request(&mut conn.message) {
                Err(_) => return Err(ConnectionError),
                Ok(None) => return Ok(()),
                Ok(Some(Request::Store(req))) => self.process_store_request(conn, req).await?,
                Ok(Some(Request::Load(req))) => self.process_load_request(conn, req).await?,
            }
        }
    }

    async fn process_store_request(
        &self,
        conn: &Connection,
        req: StoreRequest,
    ) -> ConnectionResult<()> {
        let storage_id = Self::get_storing_shard_id(&req.key);
        self.0
            .container
            .map_single(storage_id, |sharded| async move {
                let mut db = sharded.instance.db.borrow_mut();
                db.insert(req.key, req.value);
            })
            .await;

        self.respond(conn, "DONE\n").await
    }

    async fn process_load_request(
        &self,
        conn: &Connection,
        req: LoadRequest,
    ) -> ConnectionResult<()> {
        let storage_id = Self::get_storing_shard_id(&req.key);
        let load_result = self
            .0
            .container
            .map_single(storage_id, |sharded| async move {
                let db = sharded.instance.db.borrow();
                db.get(&req.key).cloned()
            })
            .await;

        match load_result {
            None => self.respond(conn, "NOTFOUND\n").await,
            Some(val) => self.respond(conn, format!("FOUND${val}\n").as_str()).await,
        }
    }

    async fn respond(&self, conn: &Connection, message: &str) -> ConnectionResult<()> {
        write(&conn.output, message)
            .await
            .map_err(|_| ConnectionError)?;
        flush_output(&conn.output)
            .await
            .map_err(|_| ConnectionError)
    }

    fn get_storing_shard_id(key: &String) -> u32 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as u32 % seastar::get_count()
    }
}
