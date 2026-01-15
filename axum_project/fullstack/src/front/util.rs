use dioxus::fullstack::http::{HeaderMap, HeaderValue};
use reqwest::header::{CACHE_CONTROL, EXPIRES, PRAGMA};

pub fn add_no_cache_headers(headers: &mut HeaderMap) {
    // 1. HTTP 1.1 표준: 캐시하지 말고, 저장하지 말고, 매번 재검증해라
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-store, must-revalidate"),
    );
    // 2. HTTP 1.0 호환성 (구형 브라우저용)
    headers.insert(PRAGMA, HeaderValue::from_static("no-cache"));
    // 3. 프록시 서버용 만료 시간 (즉시 만료)
    headers.insert(EXPIRES, HeaderValue::from_static("0"));
}
