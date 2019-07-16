pub fn get_content_length(headers: &http::HeaderMap) -> Option<usize> {
    headers.get(http::header::CONTENT_LENGTH).and_then(|h| {
        h.to_str()
            .map_err(|_| ())
            .and_then(|hv| hv.parse::<u64>().map(|l| l as usize).map_err(|_| ()))
            .ok()
    })
}
