use std::{collections::HashMap, fmt::Write};

use axum::{
    Form, Json, Router,
    body::Bytes,
    extract::{Multipart, Path, Query},
    routing::{get, post},
};
use serde::Deserialize;

async fn user() -> &'static str {
    "User get\n"
}

// url 경로상 데이터를 받는 방법
async fn path_query(Path((id, name)): Path<(i32, String)>) -> String {
    format!("{} : {}\n", id, name)
}

// url 파라메터상 데이터를 받는 방법
// HashMap으로 받는다면 type이 고정이라서 보통 Deserialize가 구현된 구조체를 이용한다.
async fn param_query_with_hashmap(Query(user): Query<HashMap<String, String>>) -> String {
    format!(
        "{} : {}\n",
        user.get("id").map_or("Id is Not Input", String::as_str),
        // .unwrap_or("Id is Not Input"),
        // .unwrap_or(&0),
        user.get("name")
            .map(String::as_str)
            .unwrap_or("Name is Not Input") // .unwrap_or(&0)
    )
}

#[derive(Deserialize)]
struct User {
    id: i32,
    name: Option<String>,
}
async fn param_query_with_struct(Query(user): Query<User>) -> String {
    format!(
        "{} : {}\n",
        user.id,
        user.name.as_deref().unwrap_or("No name")
    )
}

// http body의 데이터를 받는 방법
async fn body_query_text(name: String) -> String {
    format!("Hello {}\n", name)
}

// Bytes로 받을 수도 있는데, 주로 큰 파일등을 받기 위해 스트림을 처리해야하기 때문
async fn body_query_bytes(name: Bytes) -> String {
    format!("Hello {}\n", String::from_utf8_lossy(&name))
}

#[derive(Deserialize)]
struct TestJson {
    name: String,
}
async fn body_query_json(Json(user): Json<TestJson>) -> String {
    format!("Hello {}\n", user.name)
}
// 헤더는 application/x-www-form-urlencoded
async fn body_query_form(Form(user): Form<TestJson>) -> String {
    format!("Hello {}\n", user.name)
}

// 파일 업로드
async fn body_query_file_upload(mut body: Multipart) -> String {
    // 다중 파일을 받는다면 스트림 순서대로 받아야한다...
    // 다행이도 key를 받을 수 있으니 분류도 쉬울 것이다
    let mut result = String::new();
    // 이렇게 하면 각 파일을 전부 읽어서 메모리에 적제한다음 사용...
    // 청크단위로 스트림 형태로 받아야한다
    // while let Ok(Some(field)) = body.next_field().await {
    while let Ok(Some(mut field)) = body.next_field().await {
        let name = field.name().unwrap_or("unknown").to_string();
        let mut bytes = 0;

        while let Ok(Some(chunk)) = field.chunk().await {
            bytes += chunk.len();
        }

        writeln!(&mut result, "{} : {}", name, bytes).unwrap();
    }

    result
}

pub fn user_route() -> Router {
    let users_router = Router::new()
        .route("/", get(param_query_with_hashmap))
        // .route("/", get(param_query_with_struct))
        .route("/{id}/{name}", get(path_query))
        .route("/hello_text", post(body_query_text))
        .route("/hello_bytes", post(body_query_bytes))
        .route("/hello_json", post(body_query_json))
        .route("/hello_form", post(body_query_form))
        .route("/hello_file", post(body_query_file_upload))
        .route("/login", get(|| async move { "User Login\n" }));

    users_router
}
