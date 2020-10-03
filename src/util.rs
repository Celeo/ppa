use aes_gcm::{
    aead::{generic_array::GenericArray, Aead, NewAead},
    Aes256Gcm,
};
use anyhow::{anyhow, Result};
use clap::arg_enum;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    process,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Entry {
    pub(crate) name: String,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) comments: String,
}

arg_enum! {
    #[derive(Debug)]
    pub enum CopyWhat {
        Username,
        Password,
    }
}

fn path_to_store() -> Result<PathBuf> {
    Ok(
        Path::new(
            &home::home_dir().ok_or_else(|| anyhow!("Could not find user's home directory"))?,
        )
        .join(".ppa.bin"),
    )
}

fn read_store() -> Result<Option<Vec<Entry>>> {
    debug!("Reading store");
    let path = path_to_store()?;
    if !path.exists() {
        debug!("Store file does not exist");
        return Ok(None);
    }
    let content = fs::read_to_string(path)?;
    // TODO decrypt
    let entries: Vec<Entry> = serde_json::from_str(&content)?;
    debug!("Read {} entries from the store", entries.len());
    Ok(Some(entries))
}

pub(crate) fn create_new(encryption_password: &str) -> Result<bool> {
    debug!("Creating new store");
    let path = path_to_store()?;
    if path.exists() {
        debug!("Store already exists");
        return Ok(false);
    }
    let content = "[]";
    // TODO encrypt
    fs::write(path, content)?;
    Ok(true)
}

pub(crate) fn load_store(encryption_password: &str) -> Vec<Entry> {
    match read_store() {
        Ok(opt) if opt.is_some() => opt.unwrap(),
        Ok(_) => {
            info!("No database located, initialize with `ppa init`");
            vec![]
        }
        Err(e) => {
            error!("Could not load store file: {}", e);
            process::exit(1);
        }
    }
}

pub(crate) fn write_store(entries: &[Entry], encryption_password: &str) -> Result<()> {
    debug!("Writing store");
    let path = path_to_store()?;
    let content = serde_json::to_string(&entries)?;
    // TODO encrypt
    fs::write(path, content)?;
    Ok(())
}
