use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use rand::Rng;
use sha2::Sha256;

pub fn encrypt(key: &[u8; 32], plaintext: &str) -> Result<String, anyhow::Error> {
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| anyhow::anyhow!("AES init error: {e}"))?;

    let mut nonce = [0u8; 12];
    rand::thread_rng().fill(&mut nonce);

    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {e}"))?;

    let mut combined = nonce.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(hex::encode(&combined))
}

pub fn decrypt(key: &[u8; 32], encrypted_hex: &str) -> Result<String, anyhow::Error> {
    let data = hex::decode(encrypted_hex).map_err(|e| anyhow::anyhow!("Invalid hex: {e}"))?;

    if data.len() < 12 {
        anyhow::bail!("Encrypted data too short");
    }

    let (nonce, ciphertext) = data.split_at(12);

    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| anyhow::anyhow!("AES init error: {e}"))?;

    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {e}"))?;

    Ok(String::from_utf8(plaintext)?)
}

#[allow(dead_code)]
pub fn sha256(data: &[u8]) -> String {
    use sha2::Digest;
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}
