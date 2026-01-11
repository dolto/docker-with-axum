#[cfg(feature = "server")]
use dioxus::fullstack::{Cookie, TypedHeader};
use dioxus::{
    fullstack::{Form, SetCookie, SetHeader},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use shared::dto::user::{ReadUser, ReqUser, Tokens};

#[derive(Serialize, Deserialize)]
struct SaveId {
    save_id: bool,
}

#[post("/api/login")]
pub async fn login_action(
    Form(req_user): Form<ReqUser>,
) -> Result<(
    SetHeader<SetCookie>,
    SetHeader<SetCookie>,
    SetHeader<SetCookie>,
)> {
    // 1. Dioxus 서버 -> 실제 백엔드 API (JSON 통신)
    let client = reqwest::Client::new();
    let api_url = "http://axum-backend-dev:8000/api/auth/login"; // 도커 서비스명 주의 (backend_api 등)
    let res = client.post(api_url).json(&req_user).send().await?;

    if res.status().is_success() {
        let tokens: Tokens = res.json().await?;

        println!("---------------------");
        println!("Tokens: {:?}", tokens);
        println!("---------------------");

        let jwt_val = format!("jwt={}; Path=/; HttpOnly", tokens.jwt);
        let refresh_val = format!("refresh={}; Path=/; HttpOnly", tokens.refresh);
        let userinfo_val = format!(
            "userinfo={}; Path=/; HttpOnly",
            serde_json::to_string(&(
                tokens.user_info,
                SaveId {
                    save_id: tokens.save_id.unwrap_or(false)
                }
            ))?
        );

        // 3. 성공 반환
        Ok((
            SetHeader::new(jwt_val)?,
            SetHeader::new(refresh_val)?,
            SetHeader::new(userinfo_val)?,
        ))
    } else {
        HttpError::unauthorized("유저 정보가 잘못되었습니다")?
    }
}

#[get("/hide_get_user_cookie", header:TypedHeader<Cookie>)]
pub async fn get_user_cookie() -> Result<(ReadUser, SaveId)> {
    let user_info: (ReadUser, SaveId) =
        serde_json::from_str(header.0.get("userinfo").unwrap_or(""))?;

    Ok(user_info)
}

#[component]
pub fn Login() -> Element {
    let user_info = use_server_future(move || async move { get_user_cookie().await })?.suspend()?;

    // *로 시그널 내용을 가져오고, &로 Copy나 Clone이 발생하지 않게 함
    let user_info = &*user_info.read();

    match user_info {
        Ok(info) => {
            let username = &info.0.username;
            rsx! {
                h1 {"Wellcom! {username}"}
                form {
                    method: "post",
                    action: "/api/logout"
                }
            }
        }
        Err(_) => {
            rsx! {
                div {
                    h1 { "Login" }
                    form {
                        method: "post",
                        action: "/api/login",
                        // [필수] input name은 서버 함수 인자 이름(username)과 일치해야 함
                        input { name: "username", placeholder: "Id" }
                        br{}
                        input { name: "password", placeholder: "Pw", r#type: "password" }
                        br{}
                        label { "SaveId" }
                        input { name: "save_id", r#type: "checkbox" }
                        input {
                            r#type: "submit",
                            value: "로그인"
                        }
                    }
                }
            }
        }
    }
}
