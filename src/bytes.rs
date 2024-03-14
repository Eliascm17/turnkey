pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.into())
}

pub fn bytes_to_hex(bytes: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    Ok(bytes.iter().map(|byte| format!("{:02x}", byte)).collect())
}
