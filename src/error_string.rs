// We derive debug so we can
//   1. `expect(msg)` and `unwrap()`, the latter being undesirable though.
//   2. return Result from `main` as `impl Termination`.
#[derive(Debug)]
pub struct Error(pub String);

// Convenience method to easily output Error, i.e. `println!("{error}")`.
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

// Convenience mapping so we can do `x?` instead of `x.map_err(|e| Error(e.to_string()))?`
// whenever `x.err()` implements `std::error::Error`.
impl<E: std::error::Error> From<E> for Error {
    fn from(e: E) -> Self {
        Self(e.to_string())
    }
}

// Convenience alias because having an error handling strategy for an application
// implies returning errors of the same type often.
pub type Result<T> = std::result::Result<T, Error>;
