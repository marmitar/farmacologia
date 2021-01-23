use openssl::symm::{Crypter, Cipher, Mode::Decrypt};
use openssl::base64;
use hex::FromHex;


const BLOCK_SIZE: usize = 16;
type BLOCK = [u8; BLOCK_SIZE];


#[derive(Copy, Clone)]
pub struct Decrypter {
    key: BLOCK,
    iv: BLOCK,
    aes: Cipher,
}

impl Decrypter {
    #[inline]
    pub fn new(key: &str, iv: &str) -> Self {
        assert_eq!(key.len(), 24);
        assert_eq!(iv.len(), 32);

        let key_vec = base64::decode_block(&key).unwrap();
        assert_eq!(key_vec.len(), BLOCK_SIZE);

        let mut key = [0; BLOCK_SIZE];
        for i in 0..BLOCK_SIZE {
            key[i] = key_vec[i];
        }

        let iv = BLOCK::from_hex(&iv).unwrap();

        let aes = Cipher::aes_128_cbc();
        assert_eq!(aes.block_size(), BLOCK_SIZE);
        assert_eq!(aes.key_len(), BLOCK_SIZE);
        assert_eq!(aes.iv_len(), Some(BLOCK_SIZE));

        return Self { key, iv, aes }
    }

    #[inline]
    fn build(&self) -> Crypter {
        let mut crypter = Crypter::new(self.aes, Decrypt, &self.key, Some(&self.iv)).unwrap();
        crypter.pad(false);

        crypter
    }

    #[inline]
    pub fn decrypt(&self, input: &[u8]) -> Vec<u8> {
        let mut out = vec![0; input.len() + BLOCK_SIZE];
        let len = self.build().update(input, &mut out).unwrap();

        out.truncate(len);
        out
    }
}
