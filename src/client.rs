use {
    crate::{
        bytes::{bytes_to_hex, hex_to_bytes},
        errors::{TurnkeyError, TurnkeyResponseError, TurnkeyResult},
        models::{
            ActivityResponse, ApiStamp, SignRawPayloadIntentV2Parameters, SignRawPayloadRequest,
        },
    },
    base64_url,
    dotenv::dotenv,
    p256::ecdsa::{signature::Signer, SigningKey},
    reqwest::Client,
    serde::Deserialize,
    solana_sdk::{pubkey::Pubkey, signature::Signature, transaction::Transaction},
    std::{env, str::FromStr},
};

/// Represents the Turnkey service client, encapsulating all necessary keys and the API client.
pub struct Turnkey {
    api_public_key: String,
    api_private_key: String,
    organization_id: String,
    example_key_info: KeyInfo,
    client: Client,
}

/// Holds the private key ID and corresponding public key for a specific operation.
#[derive(Clone)]
pub struct KeyInfo {
    private_key_id: String,
    public_key: Pubkey,
}

/// Enumerates the selectable keys for operations, distinguishing by their use case.
pub enum KeySelector {
    ExampleKey,
    // other key info variants depending on what other keys you need to sign with
}

impl Turnkey {
    /// Creates a new instance of the Turnkey client.
    ///
    /// # Examples
    ///
    /// ```
    /// use turnkey::client::Turnkey;
    ///
    /// let turnkey_client = Turnkey::new();
    /// ```
    pub fn new() -> TurnkeyResult<Self> {
        dotenv().ok();

        Ok(Self {
            api_public_key: env::var("TURNKEY_API_PUBLIC_KEY")?,
            api_private_key: env::var("TURNKEY_API_PRIVATE_KEY")?,
            organization_id: env::var("TURNKEY_ORGANIZATION_ID")?,
            example_key_info: KeyInfo {
                private_key_id: env::var("TURNKEY_EXAMPLE_PRIVATE_KEY_ID")?,
                public_key: Pubkey::from_str(&env::var("TURNKEY_EXAMPLE_PUBLIC_KEY")?)?,
            },
            client: Client::new(),
        })
    }

    /// Retrieves the key information associated with the specified `KeySelector`.
    ///
    /// Returns the key information, including the private key ID and the public key,
    /// based on the specified selector. Each `KeySelector` variant corresponds to a
    /// distinct key pair within the `Turnkey` structure, facilitating access to
    /// specific cryptographic keys as needed.
    ///
    /// # Arguments
    ///
    /// * `selector` - The `KeySelector` variant indicating the key information to retrieve.
    fn select_key(&self, selector: KeySelector) -> &KeyInfo {
        match selector {
            KeySelector::ExampleKey => &self.example_key_info,
            // add more variants here as the number of keys you're managing for signing grows
        }
    }

    /// Creates a digital stamp for a given message.
    ///
    /// This method signs a given message with a private API key, generates a
    /// signature, and constructs a digital stamp containing the signature,
    /// the public API key, and the signature scheme. The digital stamp is
    /// then serialized, base64-url encoded, and returned.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to be signed and stamped.
    ///
    fn stamp(&self, message: &str) -> TurnkeyResult<String> {
        let private_api_key_bytes = hex_to_bytes(&self.api_private_key)?;
        let signing_key = SigningKey::from_bytes(&private_api_key_bytes)?;

        let signature = signing_key.sign(message.as_bytes());
        let signature_der = signature.to_der().to_bytes();
        let signature_hex = bytes_to_hex(&signature_der)?;

        let stamp = ApiStamp {
            public_key: self.api_public_key.to_string(),
            signature: signature_hex,
            scheme: "SIGNATURE_SCHEME_TK_API_P256".into(),
        };

        let json_stamp = serde_json::to_string(&stamp)?;
        let encoded_stamp = base64_url::encode(&json_stamp);

        Ok(encoded_stamp)
    }

    /// Signs a transaction using the specified key information.
    ///
    /// Asynchronously signs the provided `transaction` using the private key associated with the
    /// selected `key_selector`. This method serializes the transaction's message, signs it, and
    /// then inserts the signature into the transaction at the appropriate index based on the
    /// public key's position in the transaction's account keys. It returns the signed transaction
    /// along with the signature.
    ///
    /// The method ensures that the specified key for signing is part of the transaction's account
    /// keys, thereby validating the transaction's integrity and authorization.
    ///
    /// # Arguments
    ///
    /// * `transaction` - A mutable reference to the transaction to be signed. The transaction
    ///   is modified in place by adding the signature.
    /// * `key_selector` - A `KeySelector` variant that specifies which private key to use for
    ///   signing the transaction. The variant determines the set of key information (private and
    ///   public keys) used in the signing process.
    ///
    pub async fn sign_transaction(
        &self,
        transaction: &mut Transaction,
        key_selector: KeySelector,
    ) -> TurnkeyResult<(Transaction, Signature)> {
        let key_info = self.select_key(key_selector);
        let serialized_message = transaction.message_data();

        // get signature
        let signature_bytes = self
            .sign_bytes(&serialized_message, key_info.private_key_id.to_string())
            .await?;
        let signature = Signature::try_from(signature_bytes.as_slice())?;

        // add signature to transaction
        let index = transaction
            .message
            .account_keys
            .iter()
            .position(|key| key == &key_info.public_key);

        match index {
            Some(i) if i < transaction.signatures.len() => {
                transaction.signatures[i] = signature;
                Ok((transaction.clone(), signature))
            }
            _ => {
                return Err(TurnkeyError::OtherError(
                    "Unknown signer or index out of bounds".into(),
                ))
            }
        }
    }

    /// Asynchronously signs a byte array with the specified private key.
    ///
    /// This method constructs a request to sign a given payload represented by `bytes` using the
    /// private key identified by `private_key_id`. It sends this request to the Turnkey API,
    /// specifying that the payload is in hexadecimal format and that no hash function is applied
    /// before signing. The method waits for the signing operation to complete and processes the
    /// response to extract the signature.
    ///
    /// The signature process involves creating a digital stamp (`x_stamp`) for the request body,
    /// sending the request to the Turnkey API's sign raw payload endpoint, and then interpreting
    /// the response to retrieve the actual signature bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The byte array to be signed, represented as a slice of bytes (`&[u8]`).
    /// * `private_key_id` - A `String` representing the identifier of the private key to use for
    ///   signing the payload.
    ///
    async fn sign_bytes(&self, bytes: &[u8], private_key_id: String) -> TurnkeyResult<Vec<u8>> {
        let sign_raw_payload_body = SignRawPayloadRequest {
            activity_type: "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD_V2".to_string(),
            timestamp_ms: chrono::Utc::now().timestamp_millis().to_string(),
            organization_id: self.organization_id.clone(),
            parameters: SignRawPayloadIntentV2Parameters {
                sign_with: private_key_id,
                payload: bytes_to_hex(bytes)?,
                encoding: "PAYLOAD_ENCODING_HEXADECIMAL".to_string(),
                hash_function: "HASH_FUNCTION_NOT_APPLICABLE".to_string(),
            },
        };

        let body = serde_json::to_string(&sign_raw_payload_body)?;
        let x_stamp = self.stamp(&body)?;

        let response = self
            .client
            .post("https://api.turnkey.com/public/v1/submit/sign_raw_payload")
            .header("Content-Type", "application/json")
            .header("X-Stamp", &x_stamp)
            .body(body)
            .send()
            .await;

        let response_body = self.process_response::<ActivityResponse>(response).await?;

        if let Some(result) = response_body.activity.result {
            if let Some(result) = result.sign_raw_payload_result {
                let concatenated_hex = format!("{}{}", result.r, result.s);
                let signature_bytes = hex_to_bytes(&concatenated_hex)?;

                return Ok(signature_bytes);
            }
        }

        return Err(TurnkeyError::OtherError(
            "Missing SIGN_RAW_PAYLOAD result".into(),
        ));
    }

    /// Processes an HTTP response, handling success and error
    /// scenarios.
    ///
    /// This method takes a `Result` from an HTTP request and attempts
    /// to deserialize the response into the specified generic type
    /// `T` on success, or into a `TurnkeyError` on failure.
    ///
    /// # Arguments
    ///
    /// * `response` - A `Result` containing either a
    ///   `reqwest::Response` or a `reqwest::Error`.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The type into which the successful response should be
    ///   deserialized. This type must implement the `Deserialize`
    ///   trait.
    ///
    /// # Returns
    ///
    /// A `TurnkeyResult<T>` which is a `Result` type that contains
    /// either the deserialized response data of type `T` on
    /// success, or a `TurnkeyError` on failure.
    ///
    /// # Errors
    ///
    /// Returns `TurnkeyError::HttpError` if there is a problem with the
    /// HTTP request itself, or `TurnkeyError::MethodError` if the
    /// API returns an error response.
    async fn process_response<T>(
        &self,
        response: Result<reqwest::Response, reqwest::Error>,
    ) -> TurnkeyResult<T>
    where
        T: for<'de> Deserialize<'de> + 'static,
    {
        match response {
            Ok(res) => {
                if res.status().is_success() {
                    // On success, deserialize the response into the
                    // expected type T
                    res.json::<T>().await.map_err(TurnkeyError::HttpError)
                } else {
                    // On failure, attempt to deserialize into the error
                    // response type
                    let error_res = res.json::<TurnkeyResponseError>().await;
                    error_res
                        .map_err(TurnkeyError::HttpError)
                        .and_then(|error| Err(TurnkeyError::MethodError(error)))
                }
            }
            Err(e) => {
                // On a reqwest error, convert it into a
                // TurnkeyError::HttpError
                Err(TurnkeyError::HttpError(e))
            }
        }
    }
}
