#[cfg(feature = "server")]
pub fn no_cache_set(headers: &mut reqwest::header::HeaderMap) {
    // HTTP 1.1 표준 캐시 금지

    use reqwest::header::{HeaderValue, CACHE_CONTROL, EXPIRES, PRAGMA};
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate"),
    );
    // HTTP 1.0 표준 (Dillo 같은 고대 유물용)
    headers.insert(PRAGMA, HeaderValue::from_static("no-cache"));
    // 만료 시간 0
    headers.insert(EXPIRES, HeaderValue::from_static("0"));
}
