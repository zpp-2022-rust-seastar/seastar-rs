// An error returned when something fails while handling requests.
// We do not care what really happened because in every case we just
// close the connection with the client.
#[derive(Debug)]
pub struct ConnectionError;

pub type ConnectionResult<T> = Result<T, ConnectionError>;
