use std::io;

pub enum Error {
    IO(io::Error),
    Content,
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IO(err)
    }
}
