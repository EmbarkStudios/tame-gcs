//! Helper facilities for calculating content digests and signing data

use crate::error::Error;
use std::fmt;

/// The supported algorithms for creating a digest of content
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DigestAlgorithm {
    Sha256,
}

/// The supported algorithms for signing payloads
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SigningAlgorithm {
    RsaSha256,
}

/// The supported key formats
pub enum Key<'a> {
    /// Unencrypted PKCS#8 RSA private key. See [ring](https://briansmith.org/rustdoc/ring/signature/struct.RsaKeyPair.html#method.from_pkcs8)
    /// for more information
    Pkcs8(&'a [u8]),
    /// Uncencrypted RSA private key that isn't wrapped in PKCS#8. See [ring](https://briansmith.org/rustdoc/ring/signature/struct.RsaKeyPair.html#method.from_der)
    /// for more information
    Der(&'a [u8]),
    /// See [ring](https://briansmith.org/rustdoc/ring/hmac/index.html) for more information.
    Hmac(&'a [u8]),
}

impl<'a> fmt::Debug for Key<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Key::Pkcs8(_) => "pkcs8",
            Key::Der(_) => "der",
            Key::Hmac(_) => "hmac",
        };

        write!(f, "{}", name)
    }
}

/// Used to calculate a digest of payloads with a specific algorithm
pub trait DigestCalulator {
    /// Calculate a digest of a block of data, the algorithm determines the size
    /// of the slice used for returning the digest
    fn digest(&self, algorithm: DigestAlgorithm, data: &[u8], output_digest: &mut [u8]);
}

/// Used to sign a block of data
pub trait Signer {
    /// Sign a block of data with the specified algorith, and a private key
    fn sign(
        &self,
        algorithm: SigningAlgorithm,
        key: Key<'_>,
        data: &[u8],
    ) -> Result<Vec<u8>, Error>;
}

/// Internal type use to grab the pieces of the service account we need for signing
#[derive(Deserialize, Debug, Clone)]
struct ServiceAccountInfo {
    /// The private key we use to sign
    private_key: String,
    /// The unique id used as the issuer of the JWT claim
    client_email: String,
}

/// Provides the details needed for signing a URL
pub trait KeyProvider {
    /// The actual key used to sign the URL
    fn key(&self) -> Key<'_>;
    /// The identifier for the key author, in GCP this is the email
    /// address of the service account
    fn authorizer(&self) -> &str;
}

/// A [GCP service account](https://cloud.google.com/iam/docs/creating-managing-service-account-keys),
/// used as a `KeyProvider` when signing URLs.
pub struct ServiceAccount {
    key: Vec<u8>,
    email: String,
}

impl ServiceAccount {
    /// Attempts to load a service account from a JSON file
    pub fn load_json_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Error> {
        let file_content = std::fs::read(path)?;
        Self::load_json(file_content)
    }

    /// Attempts to load a service account from a JSON byte slice
    pub fn load_json<B: AsRef<[u8]>>(json_data: B) -> Result<Self, Error> {
        let info: ServiceAccountInfo = serde_json::from_slice(json_data.as_ref())?;

        let key_string = info
            .private_key
            .split("-----")
            .nth(2)
            .ok_or_else(|| Error::KeyRejected("invalid key format".to_owned()))?;

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

/// Implements `DigestCalculator` via [`ring`](https://briansmith.org/rustdoc/ring/digest/index.html)
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

/// Implements `Signer` via [`ring`](https://briansmith.org/rustdoc/ring/signature/index.html)
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
                    Key::Pkcs8(key) => ring::signature::RsaKeyPair::from_pkcs8(key),
                    Key::Der(key) => ring::signature::RsaKeyPair::from_der(key),
                    Key::Hmac(_) => {
                        return Err(Error::KeyRejected(
                            "HMAC cannot be used with RSA signing".to_owned(),
                        ))
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

#[cfg(test)]
mod test {
    #[test]
    fn loads_svc_account() {
        use super::KeyProvider;

        let acct = super::ServiceAccount::load_json_file("./tests/test_account.json").unwrap();

        match acct.key() {
            super::Key::Pkcs8(_) => {}
            key => panic!("invalid key format {:?}", key),
        }

        assert_eq!(
            acct.authorizer(),
            "real-address@very-good-project-id.iam.gserviceaccount.com"
        );
    }
}
