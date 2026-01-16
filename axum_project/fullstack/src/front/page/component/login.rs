use dioxus::fullstack::{Form, body::Body, extract::State, http::HeaderValue, response::Response};
use dioxus::prelude::*;

#[cfg(feature = "server")]
use dioxus::fullstack::{Cookie, TypedHeader};
#[cfg(feature = "server")]
use reqwest::header::{LOCATION, SET_COOKIE};
use serde::{Deserialize, Serialize};

use crate::front::Route;
use crate::front::util::add_no_cache_headers;

#[cfg(feature = "server")]
use crate::resources::dto::fullstack_extension::AppDatabase;
#[cfg(feature = "server")]
use crate::resources::dto::fullstack_extension::AppExtension;
use crate::resources::dto::user::ReqUser;
#[cfg(feature = "server")]
use crate::utils::errors::AppError;

#[component]
pub fn Login() -> Element {
    // 처음에만 sync 이후에는 비동기
    let LoginInfo {
        username,
        save_id,
        is_login,
    } = use_loader(|| get_user_info_from_cookie())?();
    let path = use_route::<Route>().to_string();

    if is_login {
        rsx! {
            p{"Wellcome {username.as_ref().unwrap()}"}
            form{
                method: "post",
                action: "/front/logout_action",
                button {"Logout"}
            }
        }
    } else {
        rsx! {
            form{
                method: "post",
                action: "/front/login_action",
                label { "Id: "
                    input {
                        name: "username",
                        placeholder: "Id",
                        value: if let Some(username) = username {"{username}"} else {""}
                    }
                }
                br {  }
                label { "Pw: "
                    input {
                        name: "password",
                        placeholder: "Pw",
                        r#type: "password"
                    }
                }
                br {  }
                label { "save id"
                    input {
                        name: "save_id",
                        r#type: "checkbox",
                        value: true,
                        checked: "{save_id}"
                    }
                }
                button { "login" }
                input {
                    name: "refere",
                    r#type: "hidden",
                    value: "{path}"
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct LoginInfo {
    username: Option<String>,
    save_id: bool,
    is_login: bool,
}
#[post("/front/login/login_info", header: TypedHeader<Cookie>)]
async fn get_user_info_from_cookie() -> Result<LoginInfo> {
    Ok(LoginInfo {
        username: header.0.get("username").map(|m| m.to_string()),
        save_id: header
            .0
            .get("save_id")
            .map(|m| m.parse::<bool>().unwrap_or(false))
            .unwrap_or(false),
        is_login: header.0.get("refresh").map(|m| m.to_string()).is_some(),
    })
}

#[cfg(feature = "server")]
// #[axum::debug_handler]
async fn login_action(
    State(state): State<AppDatabase>,
    Form(req_user): axum::Form<ReqUser>,
) -> Result<Response<Body>, AppError> {
    use crate::router::api::auth::*;

    println!("token check start 1");
    let token = login(State(state.0), axum::Json(req_user.clone())).await?;
    println!("token check end");

    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::SEE_OTHER;

    let header = response.headers_mut();

    let jwt_val = format!("jwt={}; Path=/; HttpOnly", token.0.jwt);
    let refresh_val = format!("refresh={}; Path=/; HttpOnly;", token.0.refresh);
    let username_val = format!("username={}; Path=/; HttpOnly", req_user.username);
    let save_id_val = format!(
        "save_id={}; Path=/; HttpOnly",
        req_user.save_id.unwrap_or(false)
    );

    add_no_cache_headers(header);

    header.insert(LOCATION, HeaderValue::from_str(&req_user.refere)?);

    header.append(SET_COOKIE, HeaderValue::from_str(&jwt_val)?);
    header.append(SET_COOKIE, HeaderValue::from_str(&refresh_val)?);
    header.append(SET_COOKIE, HeaderValue::from_str(&username_val)?);
    header.append(SET_COOKIE, HeaderValue::from_str(&save_id_val)?);

    Ok(response)
}

// #[post("/front/logout/action", header: TypedHeader<Cookie>)]
#[cfg(feature = "server")]
// #[axum::debug_handler]
async fn logout_action(
    header: axum_extra::TypedHeader<axum_extra::headers::Cookie>,
) -> Result<Response<Body>, AppError> {
    let save_id = header.0.get("save_id").map(|m| m.parse::<bool>());

    let save_id = if let Some(Ok(save_id)) = save_id {
        save_id
    } else {
        false
    };

    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::SEE_OTHER;
    let res_header = response.headers_mut();

    for (key, _) in header.iter() {
        if (key == "username" || key == "save_id") && save_id {
            continue;
        }

        res_header.append(
            SET_COOKIE,
            HeaderValue::from_str(format!("{}=; Path=/; HttpOnly; Max-Age=0;", key).as_str())?,
        );
    }
    res_header.append(LOCATION, HeaderValue::from_str("/")?);
    add_no_cache_headers(res_header);

    Ok(response)
}

#[cfg(feature = "server")]
pub fn init_router(aex: AppExtension) -> axum::Router {
    // nest front 할 예정
    axum::Router::new()
        .route("/login_action", axum::routing::post(login_action))
        .route("/logout_action", axum::routing::post(logout_action))
        .with_state(aex.db.clone())
}
