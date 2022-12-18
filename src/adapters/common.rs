use hmac::digest::generic_array::{typenum::consts::U32, GenericArray};
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub fn hmac_sha256(key: &[u8], data: &[u8]) -> GenericArray<u8, U32> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(data);
    mac.finalize().into_bytes()
}
