use std::fmt;

use reqwest::Error as ReqwestError;
use serde::Deserialize;

/// A specialized `Result` type for `Turnkey` operations.
///
/// This type is used throughout the `Turnkey` API for methods that can
/// result in a `TurnkeyError`.
pub type TurnkeyResult<T> = std::result::Result<T, TurnkeyError>;

/// Represents the errors that can occur in the `Turnkey` API.
///
/// This enum captures different types of errors that the `Turnkey` client
/// might encounter, such as method-specific errors or HTTP request
/// errors.
#[derive(Debug)]
pub enum TurnkeyError {
    /// Represents an error specific to a `Turnkey` API method.
    ///
    /// This variant is used when the `Turnkey` API returns an error
    /// response.
    MethodError(TurnkeyResponseError),

    /// Represents an HTTP request error.
    ///
    /// This variant is used for errors encountered during the HTTP
    /// request process, such as network issues or invalid
    /// responses.
    HttpError(ReqwestError),

    OtherError(String),
}

#[derive(Deserialize, Debug, Clone)]
pub struct TurnkeyResponseError {
    pub code: u32,
    pub message: String,
    pub details: Vec<ErrorDetail>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ErrorDetail {
    #[serde(rename = "@type")]
    pub type_field: String,
    #[serde(rename = "fieldViolations")]
    pub field_violations: Vec<FieldViolation>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FieldViolation {
    pub field: String,
    pub description: String,
}

impl fmt::Display for TurnkeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TurnkeyError::MethodError(e) => write!(f, "{}", e),
            TurnkeyError::HttpError(e) => write!(f, "HTTP error: {}", e),
            TurnkeyError::OtherError(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl fmt::Display for TurnkeyResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Error Code: {}, Message: {}", self.code, self.message)?;
        for detail in &self.details {
            writeln!(f, "Detail: {}", detail.type_field)?;
            for violation in &detail.field_violations {
                writeln!(
                    f,
                    "  Field: {}, Description: {}",
                    violation.field, violation.description
                )?;
            }
        }
        Ok(())
    }
}
