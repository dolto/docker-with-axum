#[cfg(feature = "server")]
use axum_extra::TypedHeader;
use dioxus::fullstack::{
    headers::Cookie,
    http::{HeaderMap, HeaderValue},
};
use dioxus::prelude::*;
use reqwest::header::{CACHE_CONTROL, EXPIRES, PRAGMA};
#[cfg(feature = "server")]
use serde::{Deserialize, Serialize};

use crate::front::Route;
#[cfg(feature = "server")]
use crate::utils::errors::AppError;

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

#[post("/front/error/msg", header: TypedHeader<Cookie>)]
pub async fn get_error_msg() -> Result<Vec<String>> {
    let mut vec = Vec::with_capacity(10);
    for (key, value) in header.iter() {
        if key.starts_with("err_msg") {
            let msg = urlencoding::decode(value)?.to_string();
            vec.push(msg);
        }
    }

    Ok(vec)
}

// And then our Outlet is wrapped in a fallback UI
#[component]
pub fn ErrorLayout() -> Element {
    let err_msg = use_loader(|| get_error_msg())?();
    let route = use_route::<Route>();

    if !err_msg.is_empty() {
        return rsx! {
            form {
                method: "post",
                action: "/front/clear_error_msg",
                input{
                    name: "path",
                    r#type:"hidden",
                    value:"{route}"
                }
                table {
                    margin: 0,
                    color: "red",
                    tbody {
                        tr{
                            width:"100%",
                            th{
                                width:"100%",
                                button {
                                    width:"100%",
                                    strong {
                                        "x"
                                    }
                                }
                            }
                        }
                        for msg in err_msg.iter() {
                            tr{
                                margin: 0,
                                td{
                                    "{msg}"
                                }
                            }
                        }
                    }
                }
            }
            Outlet::<Route>{}
        };
    }
    rsx! {
        Outlet::<Route>{}
    }
}

#[cfg(feature = "server")]
#[derive(Serialize, Deserialize)]
pub struct Refere {
    path: String,
}
#[cfg(feature = "server")]
#[axum::debug_handler]
async fn clear_error_msg(
    headers: axum_extra::TypedHeader<axum_extra::headers::Cookie>,
    axum::Form(refere): axum::Form<Refere>,
) -> Result<axum::http::Response<axum::body::Body>, AppError> {
    use axum::{body::Body, http::Response};
    use reqwest::header::{LOCATION, SET_COOKIE};

    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::SEE_OTHER;
    let res_header = response.headers_mut();
    res_header.insert(LOCATION, HeaderValue::from_str(&refere.path)?);

    for (key, _) in headers.iter() {
        if key.starts_with("err_msg") {
            res_header.append(
                SET_COOKIE,
                HeaderValue::from_str(format!("{}=; Path=/; HttpOnly; Max-Age=0;", key).as_str())?,
            );
        }
    }

    Ok(response)
}

#[cfg(feature = "server")]
pub fn init_router() -> axum::Router {
    axum::Router::new().route("/clear_error_msg", axum::routing::post(clear_error_msg))
}
