//! Windows DPAPI wrapper. The GitHub token is only ever stored as a
//! `CryptProtectData` blob (base64) inside `config.toml`, bound to the user
//! account that created it.

use base64::{engine::general_purpose::STANDARD, Engine as _};
use windows::core::PCWSTR;
use windows::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPT_INTEGER_BLOB,
};

/// DPAPI entropy. It binds the blob to this application; it is not a secret.
const ENTROPY: &[u8] = b"GitTrackWallpaper/v1";
/// Entropy used before the 0.1.0 rename. Read-only fallback so an existing
/// token keeps working after the upgrade; the next save re-encrypts with the
/// current entropy.
const LEGACY_ENTROPY: &[u8] = b"GitWallpaper/v1";

fn blob_from(bytes: &mut [u8]) -> CRYPT_INTEGER_BLOB {
    CRYPT_INTEGER_BLOB {
        cbData: bytes.len() as u32,
        pbData: bytes.as_mut_ptr(),
    }
}

fn take_blob(blob: CRYPT_INTEGER_BLOB) -> Vec<u8> {
    let out = if blob.pbData.is_null() || blob.cbData == 0 {
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(blob.pbData, blob.cbData as usize).to_vec() }
    };
    crate::win_api::local_free(blob.pbData);
    out
}

/// Encrypt a secret; empty input yields an empty string (no blob for "unset").
pub fn encrypt(plain: &str) -> Result<String, String> {
    protect(plain, ENTROPY)
}

fn protect(plain: &str, entropy_bytes: &[u8]) -> Result<String, String> {
    if plain.is_empty() {
        return Ok(String::new());
    }
    let mut input = plain.as_bytes().to_vec();
    let mut entropy = entropy_bytes.to_vec();
    let blob_in = blob_from(&mut input);
    let blob_entropy = blob_from(&mut entropy);
    let mut blob_out = CRYPT_INTEGER_BLOB::default();
    unsafe {
        CryptProtectData(
            &blob_in,
            PCWSTR::null(),
            Some(&blob_entropy),
            None,
            None,
            0,
            &mut blob_out,
        )
    }
    .map_err(|e| format!("CryptProtectData: {e}"))?;
    let out = take_blob(blob_out);
    if out.is_empty() {
        return Err("CryptProtectData returned an empty blob".into());
    }
    Ok(STANDARD.encode(out))
}

/// Decrypt a stored blob. Tries the current entropy first, then the pre-rename
/// one, so upgrading from 0.0.x never costs the user a re-pasted token.
pub fn decrypt(b64: &str) -> Result<String, String> {
    if b64.trim().is_empty() {
        return Ok(String::new());
    }
    match unprotect(b64, ENTROPY) {
        Ok(plain) => Ok(plain),
        Err(current) => unprotect(b64, LEGACY_ENTROPY).map_err(|_| current),
    }
}

fn unprotect(b64: &str, entropy_bytes: &[u8]) -> Result<String, String> {
    let mut raw = STANDARD
        .decode(b64.trim())
        .map_err(|e| format!("base64 token: {e}"))?;
    let mut entropy = entropy_bytes.to_vec();
    let blob_in = blob_from(&mut raw);
    let blob_entropy = blob_from(&mut entropy);
    let mut blob_out = CRYPT_INTEGER_BLOB::default();
    unsafe {
        CryptUnprotectData(
            &blob_in,
            None,
            Some(&blob_entropy),
            None,
            None,
            0,
            &mut blob_out,
        )
    }
    .map_err(|e| format!("CryptUnprotectData: {e}"))?;
    let out = take_blob(blob_out);
    String::from_utf8(out).map_err(|e| format!("token is not valid UTF-8: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        // Runs on the build machine under the current user; DPAPI is available
        // in a normal (non-service) session.
        let secret = "ghp_example_token_value";
        let enc = encrypt(secret).expect("encrypt");
        assert!(!enc.is_empty());
        assert_ne!(enc, secret);
        assert_eq!(decrypt(&enc).expect("decrypt"), secret);
    }

    #[test]
    fn legacy_entropy_blob_still_decrypts() {
        // A blob written by the pre-rename build must survive the upgrade; the
        // token was re-pasted often enough already.
        let secret = "ghp_from_the_old_build";
        let legacy = protect(secret, LEGACY_ENTROPY).expect("encrypt with legacy entropy");
        assert_eq!(decrypt(&legacy).expect("decrypt legacy blob"), secret);
    }

    #[test]
    fn empty_is_empty() {
        assert_eq!(encrypt("").unwrap(), "");
        assert_eq!(decrypt("").unwrap(), "");
    }
}
