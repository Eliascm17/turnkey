use {
    base64::{engine::general_purpose::URL_SAFE_NO_PAD, prelude::*},
    rand::{rngs::OsRng, RngCore},
    std::error::Error,
};

pub fn get_random_bytes(length: usize) -> Vec<u8> {
    let mut rng = OsRng;
    let mut bytes = vec![0u8; length];
    rng.fill_bytes(&mut bytes);
    bytes
}

pub fn bytes_to_base64_url(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.into())
}

pub fn bytes_to_hex(bytes: &[u8]) -> Result<String, Box<dyn Error>> {
    Ok(bytes.iter().map(|byte| format!("{:02x}", byte)).collect())
}
