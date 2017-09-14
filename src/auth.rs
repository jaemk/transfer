use errors::*;
use ring::rand::{self, SecureRandom};
use ring::constant_time;
use crypto::bcrypt;


pub fn new_salt() -> Result<Vec<u8>> {
    const SALT_SIZE: usize = 16;
    let mut salt = vec![0u8; SALT_SIZE];
    let rng = rand::SystemRandom::new();
    rng.fill(&mut salt)?;
    Ok(salt)
}


pub fn bcrypt_hash(bytes: &[u8], salt: &[u8]) -> Result<Vec<u8>> {
    let slen = salt.len();
    let blen = bytes.len();
    if !(slen == 16 && (0 < blen && blen <= 72)) {
        bail_fmt!(ErrorKind::InvalidHashArgs,
                  "Expected salt size (16) and bytes size (1-72), got: salt({}), bytes({})",
                  salt.len(), bytes.len())
    }
    const COST: u32 = 10;
    const OUTPUT_SIZE: usize = 24;
    let mut hash = vec![0u8; OUTPUT_SIZE];
    bcrypt::bcrypt(COST, salt, bytes, &mut hash);
    Ok(hash)
}


pub fn eq(a: &[u8], b: &[u8]) -> Result<()> {
    constant_time::verify_slices_are_equal(a, b)?;
    Ok(())
}

