//! Facilities for [signed URLs](https://cloud.google.com/storage/docs/access-control/signed-urls),

use crate::{error::Error, signing, types::ObjectIdentifier};
use percent_encoding as perc_enc;
use std::borrow::Cow;
use url::Url;

/// A generator for [signed URLs](https://cloud.google.com/storage/docs/access-control/signed-urls),
/// which can be used to grant temporary access to specific storage
/// resources even if the client making the request is not otherwise
/// logged in or normally able to access to the storage resources in question.
///
/// This implements the [V4 signing process](https://cloud.google.com/storage/docs/access-control/signing-urls-manually)
pub struct UrlSigner<D, S> {
    digester: D,
    signer: S,
}

#[cfg(feature = "signing")]
impl UrlSigner<signing::RingDigest, signing::RingSigner> {
    /// Creates a [`UrlSigner`] implemented via `ring`
    pub fn with_ring() -> UrlSigner<signing::RingDigest, signing::RingSigner> {
        UrlSigner::new(signing::RingDigest, signing::RingSigner)
    }
}

impl<D, S> UrlSigner<D, S>
where
    D: signing::DigestCalulator,
    S: signing::Signer,
{
    /// Creates a new [`UrlSigner`] from a [`DigestCalculator`] implementation
    /// capable of generating SHA256 digests of buffers, and a `Signer`
    /// capable of doing RSA-SHA256 encryption. You may implement these
    /// on your own using whatever crates you prefer, or you can use the
    /// `signing` feature which will use the excellent `ring` crate
    /// to provide implementations.
    pub fn new(digester: D, signer: S) -> Self {
        Self { digester, signer }
    }

    /// Generates a new signed url for the specified resource, using a key
    /// provider. Note that this operation is entirely local, so though this
    /// may succeed in generating a url, the actual operation using it may fail
    /// if the account used to sign the URL does not have sufficient permissions
    /// for the resource. For example, if you provided a GCP service account
    /// that had `devstorage.read_only` permissions for the bucket/object, this method
    /// would succeed in generating a signed url for a `POST` operation, but the actual
    /// `POST` using that url would fail as the account does not itself have permissions
    /// for the `POST` operation.
    pub fn generate<'a, K, OID>(
        &self,
        key_provider: &K,
        id: &OID,
        optional: SignedUrlOptional<'_>,
    ) -> Result<Url, Error>
    where
        K: signing::KeyProvider,
        OID: ObjectIdentifier<'a>,
    {
        // This is apparently the maximum expiration duration
        const SEVEN_DAYS: u64 = 7 * 24 * 60 * 60;
        if optional.duration.as_secs() > SEVEN_DAYS {
            return Err(Error::TooLongExpiration {
                requested: optional.duration.as_secs(),
                max: SEVEN_DAYS,
            });
        }

        // First, create the canonical request, as described here
        // https://cloud.google.com/storage/docs/authentication/canonical-requests
        //
        // HTTP_VERB
        // PATH_TO_RESOURCE
        // CANONICAL_QUERY_STRING
        // CANONICAL_HEADERS
        let mut signed_url =
            Url::parse("https://storage.googleapis.com").map_err(Error::UrlParse)?;

        // https://cloud.google.com/storage/docs/authentication/canonical-requests#about-resource-path
        let resource_path = format!(
            "/{}/{}",
            perc_enc::percent_encode(id.bucket().as_ref(), crate::util::PATH_ENCODE_SET),
            perc_enc::percent_encode(id.object().as_ref(), crate::util::PATH_ENCODE_SET),
        );

        signed_url.set_path(&resource_path);

        let mut headers = optional.headers;

        // `host` is always required
        headers.insert(
            http::header::HOST,
            http::header::HeaderValue::from_static("storage.googleapis.com"),
        );

        // Eliminate duplicate header names by creating one header name with a comma-separated list of values.
        // Be sure there is no whitespace between the values, and be sure that the order of the comma-separated
        // list matches the order that the headers appear in your request.
        let headers = {
            let mut hdrs = Vec::with_capacity(headers.keys_len());
            for key in headers.keys() {
                let vals_size = headers
                    .get_all(key)
                    .iter()
                    .fold(0, |acc, v| acc + v.len() + 1)
                    - 1;
                let mut key_vals = String::with_capacity(vals_size);
                for (i, val) in headers.get_all(key).iter().enumerate() {
                    if i > 0 {
                        key_vals.push(',');
                    }

                    key_vals.push_str(
                        val.to_str()
                            .map_err(|_err| Error::OpaqueHeaderValue(val.clone()))?,
                    );
                }

                // Make all header names lowercase.
                hdrs.push((key.as_str().to_lowercase(), key_vals));
            }

            // Sort all headers by header name using a lexicographical sort by code point value.
            hdrs.sort();
            hdrs
        };

        let signed_headers = {
            let signed_size =
                headers.iter().fold(0, |acc, (name, _)| acc + name.len()) + headers.len() - 1;
            let mut names = String::with_capacity(signed_size);

            for (i, name) in headers.iter().map(|(name, _)| name).enumerate() {
                if i > 0 {
                    names.push(';');
                }

                names.push_str(name);
            }

            assert_eq!(signed_size, names.capacity());
            names
        };

        let timestamp = time::OffsetDateTime::now_utc();

        // The date and time the signed URL became usable, in the ISO 8601 basic format YYYYMMDD'T'HHMMSS'Z'.
        let request_timestamp = {
            let year = timestamp.year();
            let month = timestamp.month() as u8;
            let day = timestamp.day();
            let hour = timestamp.hour();
            let minute = timestamp.minute();
            let second = timestamp.second();

            format!("{year:04}{month:02}{day:02}T{hour:02}{minute:02}{second:02}Z")
        };
        // YYYYMMDD
        let datestamp = &request_timestamp[..8];

        // https://cloud.google.com/storage/docs/access-control/signed-urls#credential-scope
        // [DATE]/[LOCATION]/storage/goog4_request
        let credential_scope = format!("{}/{}/storage/goog4_request", datestamp, optional.region);
        // service account email (or HMAC key)/scope
        let credential_param = format!("{}/{}", key_provider.authorizer(), credential_scope);

        let expiration = optional.duration.as_secs().to_string();

        let mut query_params = optional.query_params;

        query_params.extend(
            [
                ("X-Goog-Algorithm", "GOOG4-RSA-SHA256"),
                ("X-Goog-Credential", &credential_param),
                ("X-Goog-Date", &request_timestamp),
                ("X-Goog-Expires", &expiration),
                ("X-Goog-SignedHeaders", &signed_headers),
            ]
            .iter()
            .map(|(k, v)| (Cow::Borrowed(*k), Cow::Borrowed(*v))),
        );

        // The parameters in the query string must be sorted by name using a lexicographical sort by code point value.
        query_params.sort();

        // Fake it till you make it!
        let canonical_query = {
            {
                let mut query_pairs = signed_url.query_pairs_mut();
                query_pairs.clear(); // Shouldn't be anything here but trust nothing

                for (key, value) in &query_params {
                    query_pairs.append_pair(key, value);
                }
            }

            signed_url.query().unwrap().to_owned()
        };

        let canonical_headers = {
            let canonical_size = headers
                .iter()
                .fold(0, |acc, kv| acc + kv.0.len() + kv.1.len())
                + headers.len() * 2;
            let mut hdrs = String::with_capacity(canonical_size);

            for (k, v) in &headers {
                hdrs.push_str(k);
                hdrs.push(':');
                hdrs.push_str(v);
                hdrs.push('\n');
            }

            assert_eq!(canonical_size, hdrs.capacity());
            hdrs
        };

        // https://cloud.google.com/storage/docs/access-control/signing-urls-manually#algorithm
        // 1. Construct canonical request
        let canonical_request = format!(
            "{verb}\n{resource}\n{query}\n{headers}\n{signed_headers}\nUNSIGNED-PAYLOAD",
            verb = optional.method,
            resource = resource_path,
            query = canonical_query,
            headers = canonical_headers,
            signed_headers = signed_headers,
        );

        // 2. Use a SHA-256 hashing function to create a hex-encoded hash value of the canonical request.
        let mut digest = [0u8; 32];
        self.digester.digest(
            signing::DigestAlgorithm::Sha256,
            canonical_request.as_bytes(),
            &mut digest,
        );

        let digest_str = crate::util::to_hex(&digest);

        // 3. Construct the string-to-sign.
        // SIGNING_ALGORITHM
        // CURRENT_DATETIME
        // CREDENTIAL_SCOPE
        // HASHED_CANONICAL_REQUEST
        let string_to_sign = format!(
            "GOOG4-RSA-SHA256\n{timestamp}\n{scope}\n{hash}",
            timestamp = request_timestamp,
            scope = credential_scope,
            hash = digest_str,
        );

        let signature = self.signer.sign(
            signing::SigningAlgorithm::RsaSha256,
            key_provider.key(),
            string_to_sign.as_bytes(),
        )?;

        let signature_str = crate::util::to_hex(&signature);

        signed_url
            .query_pairs_mut()
            .append_pair("X-Goog-Signature", signature_str.as_str());

        // 4. Profit!
        Ok(signed_url)
    }
}

/// Optional parameters that can be used to tweak url signing
pub struct SignedUrlOptional<'a> {
    /// The HTTP method for the request to sign. Defaults to 'GET'.
    pub method: http::Method,
    /// The lifetime of the signed URL, as measured from the DateTime of the
    /// signed URL creation. Defaults to 1 hour.
    pub duration: std::time::Duration,
    /// Additional headers in the request
    pub headers: http::HeaderMap,
    /// The region where the resource for which the signed url is being
    /// created is for. Defaults to "auto".
    pub region: &'a str,
    /// Additional query paramters in the request
    pub query_params: Vec<(Cow<'a, str>, Cow<'a, str>)>,
}

impl<'a> Default for SignedUrlOptional<'a> {
    fn default() -> Self {
        Self {
            method: http::Method::GET,
            duration: std::time::Duration::from_secs(60 * 60),
            headers: http::HeaderMap::default(),
            region: "auto",
            query_params: Vec::new(),
        }
    }
}
