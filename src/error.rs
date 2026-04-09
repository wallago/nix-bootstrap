use thiserror::Error as ThisError;

/// Custom error type.
#[derive(Debug, ThisError)]
pub enum Error {
    /// Error that may occur while tracing system calls.
    #[error("Tracing system call error: `{0}`")]
    TraceError(String),
}

/// Type alias for the standard [`Result`] type.
pub type Result<T> = std::result::Result<T, Error>;

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use pretty_assertions::assert_eq;
//     use std::io::Error as IoError;

//     #[test]
//     fn test_error() {
//         let message = "your change your nix config!";
//         let error = Error::from(IoError::other(message));
//         assert_eq!(format!("IO error: `{message}`"), error.to_string());
//         assert_eq!(
//             format!("\"IO error: `{message}`\""),
//             format!("{:?}", error.to_string())
//         );
//     }
// }
