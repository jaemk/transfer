use std::io;
use std::net;

pub mod helpers {
    use super::*;

    pub fn reject<T: Into<Error>>(e: T) -> warp::Rejection {
        warp::reject::custom(e.into())
    }

    pub fn internal<T: AsRef<str>>(s: T) -> self::Error {
        Error::from_kind(ErrorKind::Internal(s.as_ref().to_string()))
    }

    pub fn bad_request<T: AsRef<str>>(s: T) -> self::Error {
        Error::from_kind(ErrorKind::BadRequest(s.as_ref().to_string()))
    }

    pub fn invalid_auth<T: AsRef<str>>(s: T) -> self::Error {
        Error::from_kind(ErrorKind::InvalidAuth(s.as_ref().to_string()))
    }

    pub fn too_large<T: AsRef<str>>(s: T) -> self::Error {
        Error::from_kind(ErrorKind::UploadTooLarge(s.as_ref().to_string()))
    }

    pub fn out_of_space<T: AsRef<str>>(s: T) -> self::Error {
        Error::from_kind(ErrorKind::OutOfSpace(s.as_ref().to_string()))
    }

    pub fn does_not_exist<T: AsRef<str>>(s: T) -> self::Error {
        Error::from_kind(ErrorKind::DoesNotExist(s.as_ref().to_string()))
    }

    pub fn multiple_records<T: AsRef<str>>(s: T) -> self::Error {
        Error::from_kind(ErrorKind::MultipleRecords(s.as_ref().to_string()))
    }
}

pub type Result<T> = std::result::Result<T, self::Error>;

pub trait Rejectable<T, E> {
    fn reject(self) -> std::result::Result<T, warp::Rejection>;
    fn reject_with(
        self,
        convert: impl Fn(E) -> warp::Rejection,
    ) -> std::result::Result<T, warp::Rejection>;
}

impl<T, E: Into<Error>> Rejectable<T, E> for std::result::Result<T, E> {
    fn reject(self) -> std::result::Result<T, warp::Rejection> {
        self.map_err(|e| warp::reject::custom(e.into()))
    }

    fn reject_with(
        self,
        convert: impl Fn(E) -> warp::Rejection,
    ) -> std::result::Result<T, warp::Rejection> {
        self.map_err(convert)
    }
}

#[derive(Debug)]
pub struct Error {
    kind: Box<ErrorKind>,
}
impl Error {
    pub fn kind(&self) -> &ErrorKind {
        self.kind.as_ref()
    }

    pub fn from_kind(kind: ErrorKind) -> Self {
        Self {
            kind: Box::new(kind),
        }
    }

    pub fn is_does_not_exist(&self) -> bool {
        matches!(self.kind(), self::ErrorKind::DoesNotExist(_))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use self::ErrorKind::*;
        match *self.kind() {
            S(ref s) => write!(f, "{}", s),
            Internal(ref s) => write!(f, "InternalError: {}", s),
            BadRequest(ref s) => write!(f, "BadRequest: {}", s),
            UploadTooLarge(ref s) => write!(f, "UploadTooLarge: {}", s),
            OutOfSpace(ref s) => write!(f, "OutOfSpace: {}", s),
            DoesNotExist(ref s) => write!(f, "DoesNotExist: {}", s),
            MultipleRecords(ref s) => write!(f, "MultipleRecords: {}", s),
            InvalidAuth(ref s) => write!(f, "InvalidAuth: {}", s),

            Io(ref e) => write!(f, "IoError: {}", e),
            Warp(ref e) => write!(f, "WarpError: {}", e),
            Json(ref e) => write!(f, "JsonError: {}", e),
            Uuid(ref e) => write!(f, "UuidError: {}", e),
            Hex(ref e) => write!(f, "HexError: {}", e),
            Postgres(ref e) => write!(f, "PostgresError: {}", e),
            ConnError(ref e) => write!(f, "ConnError: {}", e),
            RingUnspecified(ref e) => write!(f, "RingUnspecified: {}", e),
            Xdg(ref e) => write!(f, "Xdg: {}", e),
            ParseAddr(ref e) => write!(f, "ParseAddr: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "transfer error"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        use self::ErrorKind::*;
        Some(match *self.kind() {
            Warp(ref e) => e,
            Json(ref e) => e,
            Io(ref e) => e,
            Uuid(ref e) => e,
            Hex(ref e) => e,
            Postgres(ref e) => e,
            ConnError(ref e) => e,
            RingUnspecified(ref e) => e,
            Xdg(ref e) => e,
            ParseAddr(ref e) => e,
            _ => return None,
        })
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    S(String),
    Internal(String),
    BadRequest(String),
    UploadTooLarge(String),
    OutOfSpace(String),
    DoesNotExist(String),
    MultipleRecords(String),
    InvalidAuth(String),

    Io(io::Error),
    Warp(warp::Error),
    Json(serde_json::Error),
    Uuid(uuid::ParseError),
    Hex(hex::FromHexError),
    Postgres(postgres::error::Error),
    ConnError(r2d2::Error),
    RingUnspecified(ring::error::Unspecified),
    Xdg(xdg::BaseDirectoriesError),
    ParseAddr(net::AddrParseError),
}

impl From<&str> for Error {
    fn from(s: &str) -> Error {
        Error {
            kind: Box::new(ErrorKind::S(s.into())),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error {
            kind: Box::new(ErrorKind::Io(e)),
        }
    }
}

impl From<warp::Error> for Error {
    fn from(e: warp::Error) -> Error {
        Error {
            kind: Box::new(ErrorKind::Warp(e)),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error {
            kind: Box::new(ErrorKind::Json(e)),
        }
    }
}

impl From<uuid::ParseError> for Error {
    fn from(e: uuid::ParseError) -> Error {
        Error {
            kind: Box::new(ErrorKind::Uuid(e)),
        }
    }
}

impl From<hex::FromHexError> for Error {
    fn from(e: hex::FromHexError) -> Error {
        Error {
            kind: Box::new(ErrorKind::Hex(e)),
        }
    }
}

impl From<postgres::error::Error> for Error {
    fn from(e: postgres::error::Error) -> Error {
        Error {
            kind: Box::new(ErrorKind::Postgres(e)),
        }
    }
}

impl From<r2d2::Error> for Error {
    fn from(e: r2d2::Error) -> Error {
        Error {
            kind: Box::new(ErrorKind::ConnError(e)),
        }
    }
}

impl From<ring::error::Unspecified> for Error {
    fn from(e: ring::error::Unspecified) -> Error {
        Error {
            kind: Box::new(ErrorKind::RingUnspecified(e)),
        }
    }
}

impl From<xdg::BaseDirectoriesError> for Error {
    fn from(e: xdg::BaseDirectoriesError) -> Error {
        Error {
            kind: Box::new(ErrorKind::Xdg(e)),
        }
    }
}

impl From<net::AddrParseError> for Error {
    fn from(e: net::AddrParseError) -> Error {
        Error {
            kind: Box::new(ErrorKind::ParseAddr(e)),
        }
    }
}
