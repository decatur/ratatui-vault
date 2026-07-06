//! The ratatui-vault library
use std::{fmt, path::Path};

use argon2::Argon2;

use chacha20poly1305::{
    Key, XChaCha20Poly1305, XNonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use rand_chacha::{
    ChaCha20Rng,
    rand_core::{RngCore, SeedableRng},
};

use crate::error_string::{Error, Result};
use crate::model::Model;

#[derive(PartialEq)]
pub(crate) struct SecretString(String);

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}***", stringify!(SecretString))
    }
}

impl SecretString {
    pub(crate) fn new(s: String) -> Self {
        // TODO: Do not panic here, as we need to reset the terminal.
        // assert!(s.len() >= 8, "Secret must have at least 8 characters");
        Self(s)
    }

    pub(crate) fn plaintext(&self) -> &str {
        &self.0
    }
}

pub(crate) fn decrypt_from_file(path: &Path, password: &SecretString) -> Result<String> {
    let model = Model::deserialize(&std::fs::read(path).unwrap()[..]);
    decrypt(password.plaintext().as_bytes().to_vec(), model)
}

fn decrypt(password: Vec<u8>, model: Model) -> Result<String> {
    let mut output_key_material = [0u8; 32]; // Can be any desired size
    Argon2::default()
        .hash_password_into(&password, &model.salt, &mut output_key_material)
        .map_err(|e| Error(e.to_string()))?;

    let key = Key::from_slice(&output_key_material);
    let cipher = XChaCha20Poly1305::new(key);

    let nonce = XNonce::from_slice(&model.nonce); //GenericArray::try_from_vec(nonce).unwrap();
    let plaintext = cipher
        .decrypt(nonce, model.ciphertext.as_ref())
        .map_err(|_| Error("Could not decrypt file with provided pass phrase".to_owned()))?;

    let plaintext = String::from_utf8(plaintext)?;
    Ok(plaintext)
}

pub(crate) fn encrypt_to_file(
    plaintext: String,
    path: &Path,
    password: &SecretString,
) -> Result<()> {
    let model = encrypt(password, plaintext)?;
    std::fs::write(path, model.serialize())?;
    Ok(())
}

fn encrypt(password: &SecretString, plaintext: String) -> Result<Model> {
    // Salt should be unique per password
    let salt = argon2::password_hash::SaltString::generate(&mut OsRng);
    let mut salt_buffer = [0u8; 32];
    let salt1 = salt
        .decode_b64(&mut salt_buffer)
        .map_err(|e| Error(e.to_string()))?;

    let mut output_key_material = [0u8; 32]; // Can be any desired size
    Argon2::default()
        .hash_password_into(
            password.plaintext().as_bytes(),
            salt1,
            &mut output_key_material,
        )
        .map_err(|e| Error(e.to_string()))?;

    let key = Key::from_slice(&output_key_material);
    let cipher = XChaCha20Poly1305::new(key);
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.into_bytes().as_ref())
        .unwrap();

    Ok(Model {
        version: 1,
        salt: salt1.to_vec(),
        nonce: nonce.to_vec(),
        ciphertext,
    })
}

const CHARSET: &str = "abcdefghijklmnopqrstuvwxyz0123456789";
const CARDINALITY: usize = CHARSET.len();

/// Generate a cryptographically secure password.
pub fn generate_password(size: usize) -> String {
    assert!(u32::MAX as usize >= CARDINALITY);

    let mut generator = ChaCha20Rng::from_os_rng();
    let mut cipher = String::new();
    for _ in 0..size {
        let w = generator.next_u32() as usize;
        // We are wasting (more than) 3 out of 4 bytes, which is ok.
        let index = w % CARDINALITY;
        let c = CHARSET.chars().nth(index).unwrap();
        cipher.push(c);
    }
    cipher
}

pub(crate) fn prompt_secret(text: &str) -> SecretString {
    SecretString::new(prompt(text))
}

pub(crate) fn prompt(text: &str) -> String {
    eprint!("{text} ");
    // stderr is not buffered
    // std::io::stderr().flush().expect("We should be able flush stdout before reading the password");

    let mut response = String::new();
    // TODO: Input is not shown when we pipe stdout.
    std::io::stdin()
        .read_line(&mut response)
        .expect("We should be able to read from stdin");

    response.trim().to_owned()
}
