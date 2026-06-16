#[derive(Debug, thiserror::Error)]
pub enum DispatchError {
    #[error("handler failed: {0}")]
    HandlerFailed(String),

    #[error("invalid event data: {0}")]
    InvalidEvent(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "disk full");
        let dispatch_error: DispatchError = io_error.into();

        assert!(matches!(dispatch_error, DispatchError::Io(_)));
        assert!(dispatch_error.to_string().contains("disk full"));
    }
}
