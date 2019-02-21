pub type EmptyResult = GenericResult<()>;
pub type GenericResult<T> = Result<T, GenericError>;
pub type GenericError = Box<::std::error::Error + Send + Sync>;

#[cfg(test)]
macro_rules! s {
    ($e:expr) => ($e.to_owned())
}

// FIXME: A temporary workaround for IntelliJ Rust plugin
macro_rules! dec {
    ($e:expr) => (::rust_decimal_macros::dec!($e))
}

macro_rules! Err {
    ($($arg:tt)*) => (::std::result::Result::Err(format!($($arg)*).into()))
}