#![doc(hidden)]

pub(crate) fn to_hex<'a>(input: &[u8], output: &'a mut [u8]) -> Option<&'a str> {
    use std::str;

    const CHARS: &[u8] = b"0123456789abcdef";

    if output.len() < input.len() * 2 {
        return None;
    }

    let mut ind = 0;

    for &byte in input {
        output[ind] = CHARS[(byte >> 4) as usize];
        output[ind + 1] = CHARS[(byte & 0xf) as usize];

        ind += 2;
    }

    unsafe { Some(str::from_utf8_unchecked(&output[0..input.len() * 2])) }
}

pub fn get_content_length(headers: &http::HeaderMap) -> Option<usize> {
    headers.get(http::header::CONTENT_LENGTH).and_then(|h| {
        h.to_str()
            .map_err(|_| ())
            .and_then(|hv| hv.parse::<u64>().map(|l| l as usize).map_err(|_| ()))
            .ok()
    })
}

#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) fn if_false(v: &bool) -> bool {
    !v
}
