mod state;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use axum::{
    Extension, Form, Json, Router,
    body::Bytes,
    debug_handler,
    extract::{Multipart, Path, Query, State},
    http::{
        HeaderMap, StatusCode,
        header::{CONTENT_TYPE, USER_AGENT},
    },
    routing::{get, post},
};
use axum_extra::{
    TypedHeader,
    headers::{ContentType, UserAgent},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt::Write;

use state::*;

// url 경로상 데이터를 받는 방법

#[debug_handler]
async fn path_query(Path((id, name)): Path<(i32, String)>) -> String {
    format!("{} : {}\n", id, name)
}

// url 파라메터상 데이터를 받는 방법
// HashMap으로 받는다면 type이 고정이라서 보통 Deserialize가 구현된 구조체를 이용한다.

#[debug_handler]
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

#[debug_handler]
async fn param_query_with_struct(Query(user): Query<User>) -> String {
    format!(
        "{} : {}\n",
        user.id,
        user.name.as_deref().unwrap_or("No name")
    )
}

// http body의 데이터를 받는 방법
// 주의할점은 body의 경우 핸들러의 마지막 매개변수이자 단 하나만 존재하도록 강제한다
// http 요청 본문을 한번만 읽어서 순서대로 적제하기 때문

#[debug_handler]
async fn body_query_text(name: String) -> String {
    format!("Hello {}\n", name)
}

// Bytes로 받을 수도 있는데, 주로 큰 파일등을 받기 위해 스트림을 처리해야하기 때문

#[debug_handler]
async fn body_query_bytes(name: Bytes) -> String {
    format!("Hello {}\n", String::from_utf8_lossy(&name))
}

#[derive(Deserialize)]
struct TestJson {
    name: String,
}

#[debug_handler]
async fn body_query_json(Json(user): Json<TestJson>) -> String {
    format!("Hello {}\n", user.name)
}
// 헤더는 application/x-www-form-urlencoded

#[debug_handler]
async fn body_query_form(Form(user): Form<TestJson>) -> String {
    format!("Hello {}\n", user.name)
}

// 파일 업로드

#[debug_handler]
async fn body_query_file_upload(mut body: Multipart) -> String {
    // 다중 파일을 받는다면 스트림 순서대로 받아야한다...
    // 다행이도 key를 받을 수 있으니 분류도 쉬울 것이다
    let mut result = String::new();
    // 청크단위로 스트림 형태로 받아야한다
    // while let Ok(Some(field)) = body.next_field().await {
    while let Ok(Some(mut field)) = body.next_field().await {
        let name = field.name().unwrap_or("unknown").to_string();
        let mut bytes = 0;

        // 이렇게 하면 각 파일을 전부 읽어서 메모리에 적제한다음 사용...
        // 큰 파일의 경우 문제가 발생할 수 있으니, 청크단위로 받아야 한다
        // let bytes = field.bytes().await.unwrap();
        while let Ok(Some(chunk)) = field.chunk().await {
            bytes += chunk.len();
        }

        writeln!(&mut result, "{} : {}", name, bytes).unwrap();
    }

    result
}

#[debug_handler]
async fn header_hello1(headers: HeaderMap) -> String {
    let user_agent = headers
        .get(USER_AGENT)
        .map(|v| v.to_str().unwrap().to_string());

    let content_type = headers
        .get(CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().to_string());

    format!(
        "User-Agent: {}, Content-Type: {}\n",
        user_agent.unwrap_or_default(),
        content_type.unwrap_or_default()
    )
}

// 이거 말고도 axum-extra crate의 TypeHeader를 이용할 수도 있는데,
// 특정 헤더를 각각 가져올 수 있는 기능이 있다
// 이렇게 하면 특정 헤더가 누락되면 에러가 발생한다

#[debug_handler]
async fn header_hello2(
    user_agent: TypedHeader<UserAgent>,
    // 누락을 방지해서 Option을 넣을 수 있다 (다른 핸들러도 마찬가지)
    content_type: Option<TypedHeader<ContentType>>,
) -> String {
    format!(
        "User-Agent: {}, Content-Type: {}\n",
        user_agent.0,
        content_type.unwrap_or(TypedHeader(ContentType::text())).0
    )
}

// 응답보내기는 IntoResponse Trait를 구현하기 때문에 가능하다
// 즉 임의로 만든 타입들에 대해선 해당 Trait를 구현해줘야한다
// 아래의 모든 코드의 리턴 타입은 전부 impl IntoResponse로 할 수 있지만
// 타입을 명확성을 위해서 그렇게 하지 않기를 권장한다

// 물론 이런식으로 구조체로 활용하지 않고,
// 평문으로 작성을 지원하는 serde_json을 이용할 수도 있지만, 타입 안전성과, 성능이슈로 사용하지 않기를 권함
#[derive(Serialize)]
struct JsonTest {
    message: String,
}

#[debug_handler]
async fn response_json() -> Json<JsonTest> {
    Json({
        JsonTest {
            message: "Hello Json!".to_string(),
        }
    })
}

// 상태코드 리턴
// StatusCode의 응답코드는 반드시 (StatusCode, T) 형식이어야 한다
// 아래 주석과 같이 단일로 사용할 수도 있지만 튜플형태라면 위 규칙을 지켜야한다

// async fn response_status_code() -> StatusCode {
#[debug_handler]
async fn response_status_code() -> (StatusCode, String) {
    (StatusCode::CREATED, "Hello StatusCode!\n".to_string())
    // StatusCode::CREATED
    // 기타 다른 문제가 발생할시 다른형식으로 응답을 보낼 수 있다
    // (StatusCode::INTERNAL_SERVER_ERROR, "Error!!\n".to_string())
}

// RestAPI 에선 주로 해더와 상태코드, 데이터를 함께 보내므로, 다음과 같이한다

#[debug_handler]
async fn response_base_rest_api() -> (TypedHeader<ContentType>, (StatusCode, String)) {
    (
        TypedHeader(ContentType::text()),
        (StatusCode::CREATED, "Hello Rest!!\n".to_string()),
    )
}

// 보내야 할 TypedHeader가 여러개일 경우, 그냥 추가해주면 된다

// async fn response_base_rest_api() -> (
//     TypedHeader<ContentType>,
//     TypedHeader<ContentLength>,
//     (StatusCode, String),
// ) {
//     (
//         TypedHeader(ContentType::text()),
//         TypedHeader(ContentLength(13)),
//         (StatusCode::CREATED, "Hello Rest!!\n".to_string()),
//     )
// }
#[debug_handler]
async fn state_base_counter(State(data): State<Arc<Mutex<Vec<i32>>>>) -> String {
    let mut data = data.lock().unwrap();
    data[0] += 1;

    format!("Hello {}Times Again!\n", data[0])
}

#[debug_handler]
async fn state_appdata_name(State(data): State<HelloAppState>) -> String {
    format!("{}\n", data.auth_token)
}

#[debug_handler]
async fn state_appdata_users(State(data): State<HelloAppState>) -> String {
    format!("{}\n", data.current_users)
}

// extension도 값은 무조건 Clone으로 가져와서 값 변경 안됨
#[debug_handler]
async fn extension_appdata_users(Extension(mut data): Extension<HelloAppState>) -> String {
    data.current_users += 1;
    format!("{}\n", data.current_users)
}

// 프록시 예제
#[derive(Deserialize)]
struct Data {
    breed: String,
    num_pics: Option<i32>,
}
#[debug_handler]
async fn hello_proxy(
    State(state): State<Arc<Mutex<HashMap<String, (Bytes, usize)>>>>,
    Json(data): Json<Data>,
) -> (StatusCode, Bytes) {
    let need_pics;
    if let Some(body) = state.lock().unwrap().get(&data.breed) {
        need_pics = if let Some(num_pics) = data.num_pics {
            num_pics
        } else {
            1
        };

        if need_pics == body.1 as i32 {
            return (StatusCode::OK, body.0.clone());
        }
    }

    let mut url = format!("https://dog.ceo/api/breed/{}/images/random", &data.breed);

    if let Some(num_pics) = data.num_pics {
        url.push_str(format!("/{}", num_pics).as_str());
    }

    let client = Client::new();
    let res = client.get(url).send().await.unwrap();

    let code = res.status().as_u16();
    let body = res.bytes().await.unwrap();

    state.lock().unwrap().insert(
        data.breed,
        (body.clone(), data.num_pics.unwrap_or(1) as usize),
    );
    (StatusCode::from_u16(code).unwrap(), body)
}
pub fn hello_router() -> Router {
    let hello_router = Router::new()
        .route("/param1", get(param_query_with_hashmap))
        .route("/param2", get(param_query_with_struct))
        .route("/path/{id}/{name}", get(path_query))
        .route("/text", post(body_query_text))
        .route("/bytes", post(body_query_bytes))
        .route("/json", post(body_query_json))
        .route("/form", post(body_query_form))
        .route("/file", post(body_query_file_upload))
        .route("/header1", get(header_hello1))
        .route("/header2", get(header_hello2))
        .route("/json_response", get(response_json))
        .route("/status_code", get(response_status_code))
        .route("/rest_api", get(response_base_rest_api))
        // 라우터에서 상태에 대한 정보를 넘기는 방법은 다음과 같다
        // 상태 관리를 위해서 가져오는 데이터
        // 물론 각 요청별로 상태를 다르게 하기 위해서 get(함수명).with_state() 도 가능하다
        // route뒤의 with_state는 마지막 with_state에 따라서 그 위의 route된 모든 함수에 대응한다
        .route("/state_count", get(state_base_counter))
        .with_state(get_base_state())
        .route("/state_app_name", get(state_appdata_name))
        .route("/state_app_users", get(state_appdata_users))
        .with_state(get_hello_app_state())
        // 권장하지 않는 방식의 상태관리법 (구버전)
        .route("/extension_users", get(extension_appdata_users))
        .layer(Extension(get_hello_app_state()))
        // 프록시 서버
        .route("/proxy", post(hello_proxy))
        .with_state(get_proxy_state());

    hello_router
}
