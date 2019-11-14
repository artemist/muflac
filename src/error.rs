use std::io;
use std::num::ParseIntError;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    UTF8(FromUtf8Error),
    ParseInt(ParseIntError),
    TooLong,
    Content,
    Reserved,
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IO(err)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Self {
        Error::UTF8(err)
    }
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Error::ParseInt(err)
    }
}
