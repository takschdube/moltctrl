use rand::Rng;

/// Generate a random hex token of the specified byte length.
/// Default is 32 bytes (64 hex chars), matching openssl rand -hex 32.
pub fn generate_token(bytes: usize) -> String {
    let mut rng = rand::thread_rng();
    let token: Vec<u8> = (0..bytes).map(|_| rng.gen::<u8>()).collect();
    hex_encode(&token)
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Generate a 32-byte (64 hex char) auth token
pub fn generate_auth_token() -> String {
    generate_token(32)
}

/// Generate a 16-byte (32 hex char) pairing key
pub fn generate_pairing_key() -> String {
    generate_token(16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token_length() {
        let token = generate_token(32);
        assert_eq!(token.len(), 64);
        let token = generate_token(16);
        assert_eq!(token.len(), 32);
    }

    #[test]
    fn test_token_is_hex() {
        let token = generate_token(32);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_tokens_are_unique() {
        let t1 = generate_token(32);
        let t2 = generate_token(32);
        assert_ne!(t1, t2);
    }
}
