use {
    p256::ecdsa::Error as EcdsaError,
    reqwest::Error as ReqwestError,
    serde::Deserialize,
    serde_json::Error as SerdeJsonError,
    solana_sdk::pubkey::ParsePubkeyError,
    std::{array::TryFromSliceError, env::VarError, error::Error, fmt},
};

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
    #[allow(private_interfaces)]
    MethodError(TurnkeyResponseError),

    /// Represents an HTTP request error.
    ///
    /// This variant is used for errors encountered during the HTTP
    /// request process, such as network issues or invalid
    /// responses.
    HttpError(ReqwestError),

    /// Represents a generic error not covered by more specific `TurnkeyError` variants.
    ///
    /// This variant is used for errors that do not fit into the predefined categories
    /// of `MethodError` or `HttpError`, such as errors from external dependencies,
    /// internal logic errors, or any other situations where a more specific error
    /// cannot be provided.
    ///
    /// The contained `String` provides a human-readable description of the error,
    /// which can be useful for logging, debugging, or displaying an error message
    OtherError(String),
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct TurnkeyResponseError {
    pub code: u32,
    pub message: String,
    pub details: Vec<ErrorDetail>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct ErrorDetail {
    #[serde(rename = "@type")]
    pub type_field: String,
    #[serde(rename = "fieldViolations")]
    pub field_violations: Vec<FieldViolation>,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct FieldViolation {
    pub field: String,
    pub description: String,
}

impl From<Box<dyn Error>> for TurnkeyError {
    fn from(error: Box<dyn Error>) -> Self {
        TurnkeyError::OtherError(error.to_string())
    }
}

impl From<VarError> for TurnkeyError {
    fn from(error: VarError) -> Self {
        TurnkeyError::OtherError(error.to_string())
    }
}

impl From<EcdsaError> for TurnkeyError {
    fn from(error: EcdsaError) -> Self {
        TurnkeyError::OtherError(format!("ECDSA error: {}", error))
    }
}

impl From<SerdeJsonError> for TurnkeyError {
    fn from(error: SerdeJsonError) -> Self {
        TurnkeyError::OtherError(format!("Serde JSON error: {}", error))
    }
}

impl From<ParsePubkeyError> for TurnkeyError {
    fn from(error: ParsePubkeyError) -> Self {
        TurnkeyError::OtherError(format!("Serde JSON error: {}", error))
    }
}

impl From<TryFromSliceError> for TurnkeyError {
    fn from(error: TryFromSliceError) -> Self {
        TurnkeyError::OtherError(format!("Signature conversion error: {}", error))
    }
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
