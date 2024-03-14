use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignRawPayloadRequest {
    #[serde(rename = "type")]
    pub activity_type: String,
    pub timestamp_ms: String,
    pub organization_id: String,
    pub parameters: SignRawPayloadIntentV2Parameters,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignRawPayloadIntentV2Parameters {
    pub sign_with: String,
    pub payload: String,
    pub encoding: String,
    pub hash_function: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActivityResponse {
    pub activity: Activity,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub id: String,
    pub organization_id: String,
    pub status: String,
    pub result: Option<ActivityResult>,
    #[serde(rename = "type")]
    pub activity_type: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActivityResult {
    pub sign_raw_payload_result: Option<SignRawPayloadResult>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SignRawPayloadResult {
    pub r: String,
    pub s: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiStamp {
    pub public_key: String,
    pub signature: String,
    pub scheme: &'static str,
}
