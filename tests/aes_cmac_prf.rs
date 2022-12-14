use openssl::error::ErrorStack;
use openssl::pkey::{Id, PKey, Private};
use openssl::sign::Signer;
use openssl::symm::Cipher;
use rust_kbkdf::{PseudoRandomFunction, PseudoRandomFunctionKey};

pub struct AesCmacKey {
    key: PKey<Private>,
}

impl AesCmacKey {
    pub fn new_from_pkey(key: PKey<Private>) -> Self {
        Self { key }
    }

    pub fn new(key: &[u8]) -> Result<Self, ErrorStack> {
        let cipher = match key.len() {
            16 => Cipher::aes_128_cbc(),
            24 => Cipher::aes_192_cbc(),
            32 => Cipher::aes_256_cbc(),
            _ => panic!("Invalid key length {}", key.len()),
        };
        let key = PKey::cmac(&cipher, key)?;
        Ok(Self::new_from_pkey(key))
    }
}

impl PseudoRandomFunctionKey for AesCmacKey {
    type KeyHandle = PKey<Private>;

    fn key_handle(&self) -> &Self::KeyHandle {
        &self.key
    }
}

#[derive(Default)]
pub struct AesCmac<'a> {
    signer: Option<Signer<'a>>,
}

impl AesCmac<'_> {
    pub fn new() -> Self {
        Self { signer: None }
    }

    fn update_internal(&mut self, msg: &[u8]) -> Result<(), ErrorStack> {
        self.signer
            .as_mut()
            .expect("update called before init")
            .update(msg)
    }

    fn finish_internal(&mut self, out: &mut [u8]) -> Result<usize, ErrorStack> {
        let signer = self.signer.take().expect("finish called before init");
        signer.sign(out)
    }
}

impl<'a> AesCmac<'a> {
    fn init_internal(&mut self, key: &'a PKey<Private>) -> Result<(), ErrorStack> {
        assert!(self.signer.is_none());
        assert_eq!(key.id(), Id::CMAC);
        self.signer = Some(Signer::new_without_digest(key)?);
        Ok(())
    }
}

impl<'a> PseudoRandomFunction<'a> for AesCmac<'a> {
    type KeyHandle = PKey<Private>;
    type PrfOutputSize = typenum::U16;
    type Error = ErrorStack;

    fn init(
        &mut self,
        key: &'a dyn PseudoRandomFunctionKey<KeyHandle = Self::KeyHandle>,
    ) -> Result<(), Self::Error> {
        self.init_internal(key.key_handle())
    }

    fn update(&mut self, msg: &[u8]) -> Result<(), Self::Error> {
        self.update_internal(msg)
    }

    fn finish(&mut self, out: &mut [u8]) -> Result<usize, Self::Error> {
        self.finish_internal(out)
    }
}
