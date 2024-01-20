use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("socket error: {0}")]
    SocketError(#[from] std::io::Error),
}
