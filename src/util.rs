use aes_gcm::{
    aead::{
        generic_array::{typenum::consts::U12, GenericArray},
        Aead, NewAead,
    },
    Aes256Gcm,
};
use anyhow::{anyhow, Result};
use clap::arg_enum;
use log::debug;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// A single entry in the store.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Entry {
    /// Name of the site/service
    pub(crate) name: String,
    /// Login username
    pub(crate) username: String,
    /// Login password
    pub(crate) password: String,
    /// Any user comments
    pub(crate) comments: String,
}

arg_enum! {
    /// Whether the user wants to copy the username or password into their clipboard.
    #[derive(Debug)]
    pub enum CopyWhat {
        Username,
        Password,
    }
}

/// Return a path to the store file, which is in the user's home directory.
fn path_to_store() -> Result<PathBuf> {
    Ok(
        Path::new(
            &home::home_dir().ok_or_else(|| anyhow!("Could not find user's home directory"))?,
        )
        .join(".ppa.bin"),
    )
}

/// Check whether the store file exists on the user's system.
pub(crate) fn store_exists() -> Result<bool> {
    Ok(path_to_store()?.exists())
}

/// Load the store into memory, decrypt, and deserialize into structs.
pub(crate) fn load_store(encryption_password: &str) -> Result<Vec<Entry>> {
    debug!("Reading store");
    let path = path_to_store()?;
    if !path.exists() {
        debug!("Store file does not exist");
        return Err(anyhow!("File does not exist: initialize with `ppa init`"));
    }

    let file_content = fs::read(path)?;
    let nonce_raw: Vec<u8> = file_content.iter().take(12).cloned().collect();
    let content_encrypted: Vec<u8> = file_content.iter().skip(12).cloned().collect();

    let key = GenericArray::from_slice(encryption_password.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce: GenericArray<u8, U12> = *GenericArray::from_slice(&nonce_raw);
    let decrypted = cipher
        .decrypt(&nonce, &content_encrypted[..])
        .map_err(|e| anyhow!("Could not decrypt store: {}", e))?;
    let decrypted_str = std::str::from_utf8(&decrypted)?;

    let entries: Vec<Entry> = serde_json::from_str(&decrypted_str)?;
    debug!("Read {} entries from the store", entries.len());
    Ok(entries)
}

/// Serialize the store, encrypt, and write to disk.
pub(crate) fn write_store(entries: &[Entry], encryption_password: &str) -> Result<()> {
    debug!("Writing store");
    let path = path_to_store()?;
    let content = serde_json::to_string(&entries)?;

    let key = GenericArray::from_slice(encryption_password.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce_raw: [u8; 12] = thread_rng().gen();
    let nonce: GenericArray<u8, U12> = *GenericArray::from_slice(&nonce_raw);
    let ciphertext = cipher
        .encrypt(&nonce, content.as_bytes())
        .map_err(|e| anyhow!("Could not encrypt: {}", e))?;
    let to_disk: Vec<u8> = nonce.iter().chain(ciphertext.iter()).cloned().collect();

    fs::write(path, to_disk)?;
    Ok(())
}
