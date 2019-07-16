use crate::error::Error;

pub enum DigestAlgorithm {
    Sha256,
}

pub enum SigningAlgorithm {
    RsaSha256,
}

pub enum Key<'a> {
    Pkcs8(&'a [u8]),
    Der(&'a [u8]),
    Hmac(&'a [u8]),
}

/// Calculate a digest of a block of data, the algorithm determines
/// the size of the slice used for returning the digest
pub trait DigestCalulator {
    fn digest(&self, algorithm: DigestAlgorithm, data: &[u8], output_digest: &mut [u8]);
}

pub trait Signer {
    fn sign(
        &self,
        algorithm: SigningAlgorithm,
        key: Key<'_>,
        data: &[u8],
    ) -> Result<Vec<u8>, Error>;
}

#[derive(Deserialize, Debug, Clone)]
struct ServiceAccountInfo {
    /// The private key we use to sign
    pub private_key: String,
    /// The unique id used as the issuer of the JWT claim
    pub client_email: String,
}

pub trait KeyProvider {
    fn key(&self) -> Key<'_>;
    fn authorizer(&self) -> &str;
}

pub struct ServiceAccount {
    key: Vec<u8>,
    email: String,
}

impl ServiceAccount {
    pub fn load_json_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Error> {
        let file_content = std::fs::read(path)?;
        Self::load_json(file_content)
    }

    pub fn load_json<B: AsRef<[u8]>>(json_data: B) -> Result<Self, Error> {
        let info: ServiceAccountInfo = serde_json::from_slice(json_data.as_ref())?;

        let key_string = info
            .private_key
            .splitn(5, "-----")
            .nth(2)
            .ok_or_else(|| Error::KeyRejected("invalid key format"))?;

        // Strip out all of the newlines
        let key_string = key_string.split_whitespace().fold(
            String::with_capacity(key_string.len()),
            |mut s, line| {
                s.push_str(line);
                s
            },
        );

        let key_bytes = base64::decode_config(key_string.as_bytes(), base64::STANDARD)
            .map_err(Error::Base64Decode)?;

        Ok(Self {
            key: key_bytes,
            email: info.client_email,
        })
    }
}

impl KeyProvider for ServiceAccount {
    fn key(&self) -> Key<'_> {
        Key::Pkcs8(&self.key)
    }

    fn authorizer(&self) -> &str {
        &self.email
    }
}

#[cfg(feature = "signing")]
pub struct RingDigest;

#[cfg(feature = "signing")]
impl DigestCalulator for RingDigest {
    fn digest(&self, algorithm: DigestAlgorithm, data: &[u8], output_digest: &mut [u8]) {
        use ring::digest;

        match algorithm {
            DigestAlgorithm::Sha256 => {
                assert_eq!(
                    output_digest.len(),
                    32,
                    "output digest has invalid length for Sha256"
                );
                let digest = digest::digest(&digest::SHA256, data);
                output_digest.copy_from_slice(digest.as_ref());
            }
        }
    }
}

#[cfg(feature = "signing")]
pub struct RingSigner;

#[cfg(feature = "signing")]
impl Signer for RingSigner {
    fn sign(
        &self,
        algorithm: SigningAlgorithm,
        key: Key<'_>,
        data: &[u8],
    ) -> Result<Vec<u8>, Error> {
        match algorithm {
            SigningAlgorithm::RsaSha256 => {
                let key_pair = match key {
                    Key::Pkcs8(key) => {
                        ring::signature::RsaKeyPair::from_pkcs8(untrusted::Input::from(key))
                    }
                    Key::Der(key) => {
                        ring::signature::RsaKeyPair::from_der(untrusted::Input::from(key))
                    }
                    Key::Hmac(_) => {
                        return Err(Error::KeyRejected("HMAC cannot be used with RSA signing"))
                    }
                }?;

                let mut signature = vec![0; key_pair.public_modulus_len()];
                let rng = ring::rand::SystemRandom::new();

                key_pair.sign(
                    &ring::signature::RSA_PKCS1_SHA256,
                    &rng,
                    data,
                    &mut signature,
                )?;

                Ok(signature)
            }
        }
    }
}
