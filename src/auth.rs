/*!
Authorization/crypto things

*/
use error;
use ring::rand::{self, SecureRandom};
use ring::{digest, constant_time};
use crypto::bcrypt;


/// Generate a new 16-byte salt for use with `bcrypt`
pub fn new_salt() -> error::Result<Vec<u8>> {
    const SALT_SIZE: usize = 16;
    let mut salt = vec![0u8; SALT_SIZE];
    let rng = rand::SystemRandom::new();
    rng.fill(&mut salt)?;
    Ok(salt)
}


/// Return the SHA256 hash of `bytes`
pub fn sha256(bytes: &[u8]) -> Vec<u8> {
    let alg = &digest::SHA256;
    let digest = digest::digest(alg, bytes);
    Vec::from(digest.as_ref())
}


/// Calculate the `bcrypt` hash of `bytes` and `salt`
pub fn bcrypt_hash(bytes: &[u8], salt: &[u8]) -> error::Result<Vec<u8>> {
    let slen = salt.len();
    let blen = bytes.len();
    if !(slen == 16 && (0 < blen && blen <= 72)) {
        return Err(error::helpers::internal(
            format!(
                "Expected salt size (16) and bytes size (1-72), got: salt({}), bytes({})",
                salt.len(), bytes.len(),
            )
        ));
    }
    const COST: u32 = 10;
    const OUTPUT_SIZE: usize = 24;
    let mut hash = vec![0u8; OUTPUT_SIZE];
    bcrypt::bcrypt(COST, salt, bytes, &mut hash);
    Ok(hash)
}


/// Constant time slice equality comparison
pub fn eq(a: &[u8], b: &[u8]) -> error::Result<()> {
    constant_time::verify_slices_are_equal(a, b)
        .map_err(|_| "Bytes differ")?;
    debug!("Auth OK");
    Ok(())
}

