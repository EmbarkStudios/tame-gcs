#![doc(hidden)]

pub(crate) fn to_hex(input: &[u8]) -> String {
    const CHARS: &[u8] = b"0123456789abcdef";

    let mut result = String::with_capacity(input.len() * 2);
    for &byte in input {
        result.push(CHARS[(byte >> 4) as usize] as char);
        result.push(CHARS[(byte & 0xf) as usize] as char);
    }

    result
}

pub fn get_content_length(headers: &http::HeaderMap) -> Option<usize> {
    headers.get(http::header::CONTENT_LENGTH).and_then(|h| {
        h.to_str()
            .map_err(|_err| ())
            .and_then(|hv| hv.parse::<u64>().map(|l| l as usize).map_err(|_err| ()))
            .ok()
    })
}

#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) fn if_false(v: &bool) -> bool {
    !v
}

pub(crate) const QUERY_ENCODE_SET: &percent_encoding::AsciiSet = &percent_encoding::CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>');

pub(crate) const PATH_ENCODE_SET: &percent_encoding::AsciiSet = &QUERY_ENCODE_SET
    .add(b'`')
    .add(b'?')
    .add(b'{')
    .add(b'}')
    .add(b'%')
    .add(b'/');

#[cfg(test)]
mod test {
    #[test]
    #[allow(unsafe_code)]
    fn converts_to_hex() {
        let expected = format!("{:x}", 1234529871u32);

        let bytes = 1234529871u32.to_be_bytes();

        assert_eq!(expected, super::to_hex(&bytes));
    }
}
