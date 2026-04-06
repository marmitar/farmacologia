use hex::FromHex;
use openssl::base64;
use openssl::symm::{Cipher, Crypter, Mode::Decrypt};

const BLOCK_SIZE: usize = 16;
type Block = [u8; BLOCK_SIZE];

#[derive(Copy, Clone)]
pub struct Decrypter {
    key: Block,
    iv: Block,
    aes: Cipher,
}

impl Decrypter {
    #[inline]
    #[must_use]
    pub fn new(key: &str, iv: &str) -> Self {
        assert_eq!(key.len(), 24, "wrong key size");
        assert_eq!(iv.len(), 32, "wrong IV size");

        let key_vec = base64::decode_block(key).expect("invalid base64 key");
        assert_eq!(key_vec.len(), BLOCK_SIZE, "wrong key size");

        let key = key_vec.try_into().expect("wrong key size");

        let iv = Block::from_hex(iv).expect("invalid hexadecimal IV");

        let aes = Cipher::aes_128_cbc();
        assert_eq!(aes.block_size(), BLOCK_SIZE, "invalid block size");
        assert_eq!(aes.key_len(), BLOCK_SIZE, "invalid key size");
        assert_eq!(aes.iv_len(), Some(BLOCK_SIZE), "invalid IV size");

        Self { key, iv, aes }
    }

    #[inline]
    fn build(&self) -> Crypter {
        let mut crypter = Crypter::new(self.aes, Decrypt, &self.key, Some(&self.iv)).expect("decryption system failed");
        crypter.pad(false);

        crypter
    }

    #[inline]
    #[must_use]
    pub fn decrypt(&self, input: &[u8]) -> Vec<u8> {
        let mut out = vec![0; input.len() + BLOCK_SIZE];
        let len = self.build().update(input, &mut out).expect("could not decrypt input");

        out.truncate(len);
        out
    }
}
