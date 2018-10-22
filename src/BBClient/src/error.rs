use failure::Fail;
use redis::RedisError;
use std::io::Error as IOError;
use zydis::Status as ZydisError;

#[derive(Fail, Debug)]
pub(in crate) enum Error {
    #[fail(display = "Disassembly error: {}", _0)]
    Disasm(#[cause] ZydisError),

    #[fail(display = "IO error: {}", _0)]
    IO(#[cause] IOError),

    #[fail(display = "Cache error: {}", _0)]
    Cache(#[cause] RedisError),

    #[fail(display = "Application error: {}", _0)]
    Application(String),
}

impl From<ZydisError> for Error {
    fn from(err: ZydisError) -> Self {
        Error::Disasm(err)
    }
}

impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        Error::IO(err)
    }
}

impl From<RedisError> for Error {
    fn from(err: RedisError) -> Self {
        Error::Cache(err)
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[macro_export]
macro_rules! application_error {
    ($msg:expr) => {
        crate::error::Error::Application(String::from($msg))
    };
}
