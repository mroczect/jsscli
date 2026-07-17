use anyhow::Context;
use rand::Rng;
use std::path::PathBuf;

pub fn salt_path() -> PathBuf {
    dirs_next::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("jsscli")
        .join(".session.salt")
}

#[allow(dead_code)]
pub fn key_path() -> PathBuf {
    dirs_next::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("jsscli")
        .join(".session.key")
}

pub fn load_or_create_key() -> Result<[u8; 32], anyhow::Error> {
    let salt_path = salt_path();
    let salt = if salt_path.exists() {
        std::fs::read(&salt_path).context("Failed to read salt file")?
    } else {
        let mut salt = vec![0u8; 32];
        rand::thread_rng().fill(&mut salt[..]);
        if let Some(parent) = salt_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&salt_path, &salt)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&salt_path, std::fs::Permissions::from_mode(0o600));
        }
        salt
    };

    let master_password = get_master_password()?;

    use hkdf::Hkdf;
    let hkdf = Hkdf::<sha2::Sha256>::new(Some(&salt), master_password.as_bytes());
    let mut key = [0u8; 32];
    hkdf.expand(b"jsscli session encryption key", &mut key)
        .expect("HKDF expansion failed");
    Ok(key)
}

fn get_master_password() -> Result<String, anyhow::Error> {
    let entry = keyring::Entry::new("jsscli", "master_key")
        .map_err(|e| anyhow::anyhow!("Keyring init error: {e}"))?;

    match entry.get_password() {
        Ok(pw) => Ok(pw),
        Err(keyring::Error::NoEntry) => {
            let new_pw = generate_master_password();
            entry
                .set_password(&new_pw)
                .context("Failed to save master key to keyring")?;
            Ok(new_pw)
        }
        Err(e) => anyhow::bail!("Failed to read master key from keyring: {e}"),
    }
}

fn generate_master_password() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                              abcdefghijklmnopqrstuvwxyz\
                              0123456789!@#$%^&*()-_=+";
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
