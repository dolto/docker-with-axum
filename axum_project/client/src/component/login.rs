#[cfg(feature = "server")]
use dioxus::fullstack::{Cookie, TypedHeader};
use dioxus::{
    fullstack::{
        body::Body,
        http::header::{LOCATION, SET_COOKIE},
        response::Response,
        Form,
    },
    prelude::*,
};
#[cfg(feature = "server")]
use reqwest::header::HeaderValue;
// use serde::{Deserialize, Serialize};
use shared::dto::user::{ReadUser, ReqUser, Tokens};

#[post("/api/login")]
async fn login_action(Form(req_user): Form<ReqUser>) -> Result<Response> {
    // 1. Dioxus 서버 -> 실제 백엔드 API (JSON 통신)
    let client = reqwest::Client::new();
    let api_url = "http://axum-backend-dev:8000/api/auth/login"; // 도커 서비스명 주의 (backend_api 등)

    println!("{:?}", req_user);
    let res = client.post(api_url).json(&req_user).send().await?;

    if res.status().is_success() {
        let tokens: Tokens = res.json().await?;

        println!("---------------------");
        println!("Tokens: {:?}", tokens);
        println!("---------------------");

        let jwt_val = format!("jwt={}; Path=/; HttpOnly", tokens.jwt);
        let refresh_val = format!("refresh={}; Path=/; HttpOnly;", tokens.refresh);
        let userinfo_val = format!(
            "userinfo={}; Path=/; HttpOnly;",
            serde_json::to_string(&tokens.user_info,)?
        );
        let saveid_val = format!(
            "saveid={}; Path=/; HttpOnly=/",
            req_user.save_id.unwrap_or(false)
        );
        let mut response = Response::new(Body::empty());
        // *response.status_mut() = StatusCode::SEE_OTHER;

        let header = response.headers_mut();
        header.append(SET_COOKIE, HeaderValue::from_str(&jwt_val)?);
        header.append(SET_COOKIE, HeaderValue::from_str(&refresh_val)?);
        header.append(SET_COOKIE, HeaderValue::from_str(&userinfo_val)?);
        header.append(SET_COOKIE, HeaderValue::from_str(&saveid_val)?);
        // header.append(LOCATION, HeaderValue::from_str("/")?);
        // 3. 성공 반환

        crate::util::no_cache_set(header);
        Ok(response)
    } else {
        HttpError::unauthorized("유저 정보가 잘못되었습니다")?
    }
}

#[get("/hide_get_user_cookie", header:TypedHeader<Cookie>)]
async fn get_user_cookie() -> Result<(ReadUser, bool, bool)> {
    let user_info: ReadUser = serde_json::from_str(header.0.get("userinfo").unwrap_or(""))?;
    let saveid = header.0.get("saveid").unwrap_or("false");

    let refreshtoken = header.0.get("refresh");

    Ok((
        user_info,
        if saveid == "true" { true } else { false },
        refreshtoken.is_some(),
    ))
}

#[post("/api/logout", req_header: TypedHeader<Cookie>)]
async fn logout_action() -> Result<Response<Body>> {
    let req = reqwest::Client::new();
    let api_url = "http://axum-backend-dev:8000/api/auth/logout";
    let refresh = req_header.0.get("refresh").unwrap_or("").to_string();
    // text/plain은 기본값이기 때문에 필요 없다고함
    let _ = req
        .post(api_url)
        // .header(CONTENT_TYPE, )
        .body(refresh)
        .send()
        .await?;

    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::SEE_OTHER;
    let header = response.headers_mut();
    let saveid = req_header.0.get("saveid").unwrap_or("false");
    for (key, v) in req_header.0.iter() {
        if saveid == "true" && (key == "userinfo" || key == "saveid") {
            continue;
        }
        println!("{}: {} is deleted", key, v);
        header.append(
            SET_COOKIE,
            HeaderValue::from_str(format!("{}=; Path=/; HttpOnly; Max-Age=0;", key).as_str())?,
        );
    }
    header.append(LOCATION, HeaderValue::from_str("/")?);
    crate::util::no_cache_set(header);

    Ok(response)
}

#[component]
pub fn Login() -> Element {
    let user_info = use_server_future(move || async move { get_user_cookie().await })?.suspend()?;

    // *로 시그널 내용을 가져오고, &로 Copy나 Clone이 발생하지 않게 함
    let user_info = &*user_info.read();

    match user_info {
        Ok((info, saveid, is_login)) => {
            let username = &info.username;
            if *is_login {
                rsx! {
                    h1 {"Wellcom! {username}"}
                    form {
                        method: "post",
                        action: "/api/logout",
                        input {
                            r#type: "submit",
                            value: "Logout"
                        }
                    }
                }
            } else if *saveid {
                need_login(Some(username.clone()))
            } else {
                need_login(None)
            }
        }
        Err(_) => need_login(None),
    }
}

fn need_login(id: Option<String>) -> Element {
    rsx! {
        div {
            h1 { "Login" }
            form {
                method: "post",
                action: "/api/login",
                // [필수] input name은 서버 함수 인자 이름(username)과 일치해야 함
                input { name: "username", placeholder: "Id",
                    value: if let Some(id) = id {"{id}"} else {""}
                }
                br{}
                input { name: "password", placeholder: "Pw", r#type: "password",
                }
                br{}
                label { "SaveId" }
                input { name: "save_id", r#type: "checkbox", value: "true" }
                input {
                    r#type: "submit",
                    value: "Login",
                }
            }
        }
    }
}
