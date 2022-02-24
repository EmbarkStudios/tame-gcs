//! Helper types for working with GCS

use crate::error::Error;
use std::borrow::Cow;

/// A wrapper around strings meant to be used as bucket names,
/// to validate they conform to [Bucket Name Requirements](https://cloud.google.com/storage/docs/naming#requirements)
#[derive(Debug)]
pub struct BucketName<'a> {
    name: Cow<'a, str>,
}

impl<'a> BucketName<'a> {
    /// Creates a [`BucketName`] without validating it, meaning
    /// that invalid names will result in API failures when
    /// requests are actually made to GCS instead.
    pub fn non_validated<S: AsRef<str> + ?Sized>(name: &'a S) -> Self {
        Self {
            name: Cow::Borrowed(name.as_ref()),
        }
    }

    /// Validates the string is a syntactically valid bucket name
    fn validate(name: &str) -> Result<(), Error> {
        let count = name.chars().count();

        // Bucket names must contain 3 to 63 characters.
        if !(3..=63).contains(&count) {
            return Err(Error::InvalidCharacterCount {
                len: count,
                min: 3,
                max: 63,
            });
        }

        let last = count - 1;

        for (i, c) in name.chars().enumerate() {
            if c.is_ascii_uppercase() {
                return Err(Error::InvalidCharacter(i, c));
            }

            match c {
                'a'..='z' | '0'..='9' => {}
                '-' | '_' => {
                    // Bucket names must start and end with a number or letter.
                    if i == 0 || i == last {
                        return Err(Error::InvalidCharacter(i, c));
                    }
                }
                c => {
                    return Err(Error::InvalidCharacter(i, c));
                }
            }
        }

        // Bucket names cannot begin with the "goog" prefix.
        if name.starts_with("goog") {
            return Err(Error::InvalidPrefix("goog"));
        }

        // Bucket names cannot contain "google" or close misspellings, such as "g00gle".
        // They don't really specify what counts as a "close" misspelling, so just check
        // the ones they say, and let the API deny the rest
        if name.contains("google") || name.contains("g00gle") {
            return Err(Error::InvalidSequence("google"));
        }

        Ok(())
    }
}

impl<'a> std::fmt::Display for BucketName<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

impl<'a> AsRef<str> for BucketName<'a> {
    fn as_ref(&self) -> &str {
        self.name.as_ref()
    }
}

impl<'a> AsRef<[u8]> for BucketName<'a> {
    fn as_ref(&self) -> &[u8] {
        self.name.as_bytes()
    }
}

impl<'a> TryFrom<&'a str> for BucketName<'a> {
    type Error = Error;

    fn try_from(n: &'a str) -> Result<Self, Self::Error> {
        Self::validate(n)?;

        Ok(Self {
            name: Cow::Borrowed(n),
        })
    }
}

impl<'a> TryFrom<String> for BucketName<'a> {
    type Error = Error;

    fn try_from(n: String) -> Result<Self, Self::Error> {
        Self::validate(&n)?;

        Ok(Self {
            name: Cow::Owned(n),
        })
    }
}

/// A wrapper for strings meant to be used as object names, to validate
/// that they follow [Object Name Requirements](https://cloud.google.com/storage/docs/naming#objectnames)
#[derive(Debug)]
pub struct ObjectName<'a> {
    name: Cow<'a, str>,
}

impl<'a> ObjectName<'a> {
    /// Creates an `ObjectName` without validating it, meaning
    /// that invalid names will result in API failures when
    /// requests are actually made to GCS instead.
    pub fn non_validated<S: AsRef<str> + ?Sized>(name: &'a S) -> Self {
        Self {
            name: Cow::Borrowed(name.as_ref()),
        }
    }

    /// Validates the string is a syntactically valid object name
    fn validate(name: &str) -> Result<(), Error> {
        // Object names can contain any sequence of valid Unicode characters, of length 1-1024 bytes when UTF-8 encoded.
        if name.is_empty() || name.len() > 1024 {
            return Err(Error::InvalidLength {
                min: 1,
                max: 1024,
                len: name.len(),
            });
        }

        // Objects cannot be named . or ...
        if name == "." || name == "..." {
            return Err(Error::InvalidPrefix("."));
        }

        #[allow(clippy::match_same_arms)]
        for (i, c) in name.chars().enumerate() {
            match c {
                // Object names cannot contain Carriage Return or Line Feed characters.
                '\r' | '\n' => {}
                // Avoid using "#" in your object names: gsutil interprets object names ending
                // with #<numeric string> as version identifiers, so including "#" in object names
                // can make it difficult or impossible to perform operations on such versioned
                // objects using gsutil (see Object Versioning and Concurrency Control).
                // Avoid using "[", "]", "*", or "?" in your object names:  gsutil interprets
                // these characters as wildcards, so including them in object names can make
                // it difficult or impossible to perform wildcard operations using gsutil.
                '#' | '[' | ']' | '*' | '?' => {}
                // Avoid using control characters that are illegal in XML 1.0 (#x7F–#x84 and #x86–#x9F):
                // these characters will cause XML listing issues when you try to list your objects.
                '\u{7F}'..='\u{84}' | '\u{86}'..='\u{9F}' => {}
                _ => {
                    continue;
                }
            }

            return Err(Error::InvalidCharacter(i, c));
        }

        // Object names cannot start with .well-known/acme-challenge.
        if name.starts_with(".well-known/acme-challenge") {
            return Err(Error::InvalidPrefix(".well-known/acme-challenge"));
        }

        Ok(())
    }
}

impl<'a> std::fmt::Display for ObjectName<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

impl<'a> AsRef<str> for ObjectName<'a> {
    fn as_ref(&self) -> &str {
        self.name.as_ref()
    }
}

impl<'a> AsRef<[u8]> for ObjectName<'a> {
    fn as_ref(&self) -> &[u8] {
        self.name.as_bytes()
    }
}

impl<'a> TryFrom<&'a str> for ObjectName<'a> {
    type Error = Error;

    fn try_from(n: &'a str) -> Result<Self, Self::Error> {
        Self::validate(n)?;

        Ok(Self {
            name: Cow::Borrowed(n),
        })
    }
}

impl<'a> TryFrom<String> for ObjectName<'a> {
    type Error = Error;

    fn try_from(n: String) -> Result<Self, Self::Error> {
        Self::validate(&n)?;

        Ok(Self {
            name: Cow::Owned(n),
        })
    }
}

impl<'a> AsRef<BucketName<'a>> for (&'a BucketName<'a>, &'a ObjectName<'a>) {
    fn as_ref(&self) -> &BucketName<'a> {
        self.0
    }
}

impl<'a> AsRef<ObjectName<'a>> for (&'a BucketName<'a>, &'a ObjectName<'a>) {
    fn as_ref(&self) -> &ObjectName<'a> {
        self.1
    }
}

pub trait ObjectIdentifier<'a> {
    fn bucket(&self) -> &BucketName<'a>;
    fn object(&self) -> &ObjectName<'a>;
}

impl<'a, T> ObjectIdentifier<'a> for T
where
    T: AsRef<BucketName<'a>> + AsRef<ObjectName<'a>>,
{
    fn bucket(&self) -> &BucketName<'a> {
        self.as_ref()
    }

    fn object(&self) -> &ObjectName<'a> {
        self.as_ref()
    }
}

/// A concrete object id which contains a valid bucket and object name
/// which fully specifies an object
pub struct ObjectId<'a> {
    pub bucket: BucketName<'a>,
    pub object: ObjectName<'a>,
}

impl<'a> ObjectId<'a> {
    pub fn new<B, O>(bucket: B, object: O) -> Result<Self, Error>
    where
        B: std::convert::TryInto<BucketName<'a>, Error = Error> + ?Sized,
        O: std::convert::TryInto<ObjectName<'a>, Error = Error> + ?Sized,
    {
        Ok(Self {
            bucket: bucket.try_into()?,
            object: object.try_into()?,
        })
    }
}

impl<'a> AsRef<BucketName<'a>> for ObjectId<'a> {
    fn as_ref(&self) -> &BucketName<'a> {
        &self.bucket
    }
}

impl<'a> AsRef<ObjectName<'a>> for ObjectId<'a> {
    fn as_ref(&self) -> &ObjectName<'a> {
        &self.object
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn disallows_too_small() {
        assert_eq!(
            BucketName::try_from("no").unwrap_err(),
            Error::InvalidCharacterCount {
                len: 2,
                min: 3,
                max: 63,
            }
        );
    }

    #[test]
    fn disallows_too_big() {
        assert_eq!(
            BucketName::try_from(
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            )
            .unwrap_err(),
            Error::InvalidCharacterCount {
                len: 64,
                min: 3,
                max: 63
            }
        );
    }

    #[test]
    fn disallows_uppercase() {
        assert_eq!(
            BucketName::try_from("uhOH").unwrap_err(),
            Error::InvalidCharacter(2, 'O')
        );
    }

    #[test]
    fn disallows_dots() {
        assert_eq!(
            BucketName::try_from("uh.oh").unwrap_err(),
            Error::InvalidCharacter(2, '.')
        );
    }

    #[test]
    fn disallows_hyphen_or_underscore_at_start() {
        assert_eq!(
            BucketName::try_from("_uhoh").unwrap_err(),
            Error::InvalidCharacter(0, '_')
        );
    }

    #[test]
    fn disallows_hyphen_or_underscore_at_end() {
        assert_eq!(
            BucketName::try_from("uhoh-").unwrap_err(),
            Error::InvalidCharacter(4, '-')
        );
    }

    #[test]
    fn disallows_goog_at_start() {
        assert_eq!(
            BucketName::try_from("googuhoh").unwrap_err(),
            Error::InvalidPrefix("goog")
        );
    }

    #[test]
    fn disallows_google_sequence() {
        assert_eq!(
            BucketName::try_from("uhohg00gleuhoh").unwrap_err(),
            Error::InvalidSequence("google")
        );
    }
}
