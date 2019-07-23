use crate::{
    JSONAPI_MIME,
    models::ConversionError,
};
use identity::ID_LEN;
use iron::{
    IronError,
    status::Status,
};
use json_api::{
    Document,
    Error,
    ErrorSource,
    ObjectConversionError,
};
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result},
};

#[derive(Debug)]
pub (crate) enum AuthError {
    MultipleData,
    NoData,
    ConversionError(ObjectConversionError),
    NoSecret,
    InvalidIdentity(ConversionError),
}

impl AuthError {
    fn detail(&self) -> String {
        match self {
            AuthError::MultipleData => 
                "Multiple data were provided when the endpoint expects exactly one".into(),
            AuthError::NoData => "Document contains no data".into(),
            AuthError::ConversionError(e) => format!("Error converting generic object ({})", e),
            AuthError::NoSecret => "No secret provided".into(),
            AuthError::InvalidIdentity(e) => format!("Conversion Error ({})", e), 
        }
    }
}

impl StdError for AuthError {}

impl Display for AuthError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Auth Error: {}", self.detail())
    }
}

impl From<AuthError> for IronError {
    fn from(e: AuthError) -> IronError {
        let status = match e {
            _ => Status::BadRequest,
        };

        let title = match e {
            AuthError::MultipleData => Some("Multiple Data".into()),
            AuthError::NoData => Some("No Data".into()),
            AuthError::ConversionError(_) => Some("Object Error".into()),
            AuthError::NoSecret => Some("No Secret".into()),
            AuthError::InvalidIdentity(_) => Some("Invalid identity".into()),
        };

        let detail = match e {
            AuthError::ConversionError(ObjectConversionError::ImproperType{ expected, got }) => 
                Some(format!("Primary data should be of type {} but is of type {} instead", 
                             expected, got)),
            AuthError::ConversionError(ObjectConversionError::FailedDeserialization(e)) =>
                Some(format!("Failed to deserialize attributes of primary data: {}", e)),
            AuthError::NoSecret => Some("A secret is required to log in and none was provided".into()),
            AuthError::InvalidIdentity(ConversionError::Base64Decode(e)) => 
                Some(format!("Failed to decode identity, base 64 invalid: {}", e)),
            AuthError::InvalidIdentity(ConversionError::BadIdLength(l)) =>
                Some(format!("Failed to decode identity, decoded identity is {} bytes long when it should be {}", l, ID_LEN)),
            _ => Some(e.detail()),
        };

        let pointer = match e {
            AuthError::MultipleData => Some("/data".into()),
            AuthError::NoData => Some("/".into()),
            AuthError::ConversionError(ObjectConversionError::ImproperType{ expected: _, got: _ }) =>
                Some("/data/type".into()),
            AuthError::ConversionError(ObjectConversionError::FailedDeserialization(_)) => 
                Some("/data/attributes".into()),
            AuthError::NoSecret => Some("/data/attributes".into()),
            AuthError::InvalidIdentity(_) => Some("/data/id".into()),
        };

        let document = Document { 
            errors: Some(vec![Error {
                status: Some(format!("{}", status.to_u16())),
                title,
                detail,
                source: pointer.map(|p| ErrorSource { pointer: Some(p), ..Default::default() }),
                ..Default::default()
            }]),
            ..Default::default()
        };

        Self::new(e, (status, serde_json::to_string(&document).unwrap(), JSONAPI_MIME.clone()))
    }
}