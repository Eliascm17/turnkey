use p256::ecdsa::{signature::Signer, SigningKey};
use sha2::Digest;

use {
    crate::{
        bytes::{bytes_to_hex, hex_to_bytes},
        errors::{TurnkeyError, TurnkeyResponseError, TurnkeyResult},
    },
    dotenv::dotenv,
    reqwest::Client,
    serde::{Deserialize, Serialize},
    solana_sdk::{pubkey::Pubkey, signature::Signature, transaction::Transaction},
    std::env,
};

pub struct Turnkey {
    api_public_key: String,
    api_private_key: String,
    private_key_id: String,
    organization_id: String,
    client: Client,
}

impl Turnkey {
    /// Creates a new instance of the Turnkey client.
    ///
    /// # Examples
    ///
    /// ```
    /// use turnkey::Turnkey;
    ///
    /// let turnkey_client = Turnkey::new();
    /// ```
    pub fn new() -> Turnkey {
        dotenv().ok();

        let api_public_key =
            env::var("TURNKEY_API_PUBLIC_KEY").expect("TURNKEY_API_PUBLIC_KEY must be set");
        let api_private_key =
            env::var("TURNKEY_API_PRIVATE_KEY").expect("TURNKEY_API_PRIVATE_KEY must be set");
        let private_key_id =
            env::var("TURNKEY_PRIVATE_KEY_ID").expect("TURNKEY_PRIVATE_KEY_ID must be set");
        let organization_id =
            env::var("TURNKEY_ORGANIZATION_ID").expect("TURNKEY_ORGANIZATION_ID must be set");

        Turnkey {
            api_public_key,
            api_private_key,
            private_key_id,
            organization_id,
            client: Client::new(),
        }
    }

    pub async fn who_am_i(&self) -> TurnkeyResult<WhoAmIResponse> {
        let body = &serde_json::json!({"organizationId": self.organization_id});
        let x_stamp = self.stamp(&body.to_string()).unwrap();

        println!("x_stamp: {}", x_stamp);

        let response = self
            .client
            .post("https://api.turnkey.com/public/v1/query/whoami")
            .header("Content-Type", "application/json")
            .header("X-Stamp", &x_stamp)
            .json(body)
            .send()
            .await;

        self.process_response::<WhoAmIResponse>(response).await
    }

    pub fn stamp(&self, message: &str) -> TurnkeyResult<String> {
        let signing_key =
            SigningKey::from_bytes(&hex_to_bytes(&self.api_private_key).unwrap()).unwrap();

        let mut hasher = sha2::Sha256::new();
        hasher.update(&message);
        let hashed_message = hasher.finalize();

        let signature = signing_key.sign(&hashed_message);
        let der_encoded_signature = signature.as_ref();
        let signature_hex = bytes_to_hex(der_encoded_signature).unwrap();

        let stamp = ApiStamp {
            public_key: self.api_public_key.to_string(),
            signature: signature_hex,
            scheme: "SIGNATURE_SCHEME_TK_API_P256".into(),
        };

        let json_stamp = serde_json::to_string(&stamp).unwrap();
        let encoded_stamp = base64::encode(json_stamp.as_bytes());

        Ok(encoded_stamp)
    }

    pub async fn sign_transaction(
        &self,
        transaction: &mut Transaction,
        public_key: &Pubkey,
    ) -> TurnkeyResult<(Transaction, Signature)> {
        let serialized_message = transaction.message_data();
        let signature_bytes = self.sign_bytes(&serialized_message).await?;

        let signature = Signature::try_from(signature_bytes.as_slice()).map_err(|_| {
            TurnkeyError::OtherError("Failed to convert signature bytes into a Signature".into())
        })?;

        let index = transaction
            .message
            .account_keys
            .iter()
            .position(|key| key == public_key);

        match index {
            Some(i) if i < transaction.signatures.len() => {
                transaction.signatures[i] = signature;
                Ok((transaction.clone(), signature))
            }
            _ => return Err(TurnkeyError::OtherError("unknown signer".into())),
        }
    }

    async fn sign_bytes(&self, bytes: &[u8]) -> TurnkeyResult<Vec<u8>> {
        let payload = bytes_to_hex(bytes).map_err(|e| TurnkeyError::OtherError(e.to_string()))?;

        let sign_raw_payload_body = SignRawPayloadRequest {
            activity_type: "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD_V2".to_string(),
            timestamp_ms: chrono::Utc::now().timestamp_millis().to_string(),
            organization_id: self.organization_id.clone(),
            parameters: SignRawPayloadIntentV2Parameters {
                sign_with: self.private_key_id.clone(),
                payload,
                encoding: "PAYLOAD_ENCODING_HEXADECIMAL".to_string(),
                hash_function: "HASH_FUNCTION_NOT_APPLICABLE".to_string(),
            },
        };

        let body = serde_json::to_string(&sign_raw_payload_body).unwrap();
        let x_stamp = self.stamp(&body).unwrap(); // probably doesn't matter but try this

        println!("body: {:#?}", body);

        let response = self
            .client
            .post("https://api.turnkey.com/public/v1/submit/sign_raw_payload")
            .header("Content-Type", "application/json")
            .header("X-Stamp", &x_stamp)
            .body(body)
            .send()
            .await;

        let response_body = self
            .process_response::<SignRawPayloadResponse>(response)
            .await?;

        if let Some(result) = response_body.activity.result {
            let concatenated_hex = format!("{}{}", result.r, result.s);
            let signature_bytes = hex_to_bytes(&concatenated_hex)
                .map_err(|e| TurnkeyError::OtherError(e.to_string()))?;

            Ok(signature_bytes)
        } else {
            Err(TurnkeyError::OtherError(
                "Missing SIGN_RAW_PAYLOAD result".into(),
            ))
        }
    }

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignRawPayloadRequest {
    #[serde(rename = "type")]
    pub activity_type: String,
    #[serde(rename = "timestampMS")]
    pub timestamp_ms: String,
    #[serde(rename = "organizationId")]
    pub organization_id: String,
    pub parameters: SignRawPayloadIntentV2Parameters,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignRawPayloadIntentV2Parameters {
    #[serde(rename = "signWith")]
    pub sign_with: String,
    pub payload: String,
    pub encoding: String,
    #[serde(rename = "hashFunction")]
    pub hash_function: String,
}

#[derive(Deserialize)]
struct SignRawPayloadResponse {
    pub activity: Activity,
}

#[derive(Deserialize)]
struct Activity {
    pub id: String,
    #[serde(rename = "organizationId")]
    pub organization_id: String,
    pub status: String,
    pub result: Option<SignRawPayloadResult>,
    #[serde(rename = "type")]
    pub activity_type: String,
}

#[derive(Deserialize)]
struct SignRawPayloadResult {
    pub r: String,
    pub s: String,
    pub v: String,
}

#[derive(Deserialize, Debug)]
pub struct WhoAmIResponse {
    #[serde(rename = "organizationId")]
    pub organization_id: String,
    #[serde(rename = "organizationName")]
    pub organization_name: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub username: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiStamp {
    public_key: String,
    signature: String,
    scheme: &'static str,
}
