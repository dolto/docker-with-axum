mod database;
mod open_api;
mod state;
use std::{collections::HashMap, sync::Arc};

use axum::{
    Extension, Form, Json, Router,
    body::Bytes,
    debug_handler,
    extract::{Multipart, Path, Query, State},
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{CONTENT_TYPE, USER_AGENT},
    },
};
use axum_extra::{
    TypedHeader,
    headers::{ContentType, UserAgent},
};
use reqwest::Client;
use sea_orm::{ColumnTrait, Condition, DatabaseConnection};
use serde::{Deserialize, Serialize};
use shared::dto::user::UserDTO;
use std::fmt::Write;
use tokio::sync::Mutex;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

use state::*;

use crate::{
    entities::users,
    router::hello::{
        database::{
            hello_delete_by_id, hello_delete_by_model, hello_delete_many, hello_insert_many,
            hello_insert_one1, hello_insert_one2, hello_select_all, hello_select_one,
            hello_update_many, hello_update_one1, hello_update_one2,
        },
        open_api::{HELLO_TAG, set_router},
    },
    utils::errors::AppError,
};

// url 경로상 데이터를 받는 방법
#[utoipa::path(
    get,
    path = "/path/{id}/{name}",
    tag = HELLO_TAG,
    params(
        ("id" = i32, Path, description = "numeric id"),
        ("name" = String, Path, description = "user name")
    ),
    responses(
        (
            status = 200,
            description = "get id(i32) name(String), and print",
            body = String,
            example = "1 : dolto"
        )
    )
)]
#[debug_handler]
async fn path_query(Path((id, name)): Path<(i32, String)>) -> String {
    format!("{} : {}\n", id, name)
}

// url 파라메터상 데이터를 받는 방법
// HashMap으로 받는다면 type이 고정이라서 보통 Deserialize가 구현된 구조체를 이용한다.
#[utoipa::path(
    get,
    path = "/param1",
    tag = HELLO_TAG,
    params(
        ("id" = Option<String>, Query, description = "user id"),
        ("name" = Option<String>, Query, description = "user name")
    ),
    responses(
        (
            status = 200,
            description = "get id(String) name(String), and print",
            body = String,
            example = "d : dolto"
        )
    )
)]
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

#[derive(Deserialize, IntoParams)]
struct User {
    id: i32,
    name: Option<String>,
}

#[utoipa::path(
    get,
    path = "/param2",
    tag = HELLO_TAG,
    params(
        User
    ),
    responses(
        (
            status = 200,
            description = "get id(String) name(String), and print",
            body = String,
            example = "d : dolto"
        )
    )
)]
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

#[utoipa::path(
    post,
    path = "/text",
    tag = HELLO_TAG,
    request_body(
        content = String,
        description = "get String",
    ),
    responses(
        (
            status = 200,
            description = "get name(String), and print",
            body = String,
            example = "Hello dolto\n"
        )
    )
)]
#[debug_handler]
async fn body_query_text(name: String) -> String {
    format!("Hello {}\n", name)
}

// Bytes로 받을 수도 있는데, 주로 큰 파일등을 받기 위해 스트림을 처리해야하기 때문

#[utoipa::path(
    post,
    path = "/bytes",
    tag = HELLO_TAG,
    request_body(
        content = String,
        description = "get Bytes",
    ),
    responses(
        (
            status = 200,
            description = "get name(Bytes), and print",
            body = String,
            example = "Hello dolto\n"
        )
    )
)]
#[debug_handler]
async fn body_query_bytes(name: Bytes) -> String {
    format!("Hello {}\n", String::from_utf8_lossy(&name))
}

#[derive(Deserialize, ToSchema)]
struct TestJson {
    name: String,
}

#[utoipa::path(
    post,
    path = "/json",
    tag = HELLO_TAG,
    request_body(
        content = TestJson,
        content_type = mime::APPLICATION_JSON.as_ref(),
        description = "get Json",
    ),
    responses(
        (
            status = 200,
            description = "get name(Json), and print",
            body = String,
            example = "Hello dolto\n"
        )
    )
)]
#[debug_handler]
async fn body_query_json(Json(user): Json<TestJson>) -> String {
    format!("Hello {}\n", user.name)
}
// 헤더는 application/x-www-form-urlencoded

#[utoipa::path(
    post,
    path = "/form",
    tag = HELLO_TAG,
    request_body(
        content = TestJson,
        content_type = mime::WWW_FORM_URLENCODED.as_ref(),
        description = "get form",
    ),
    responses(
        (
            status = 200,
            description = "get name(Form), and print",
            body = String,
            example = "Hello dolto\n"
        )
    )
)]
#[debug_handler]
async fn body_query_form(Form(user): Form<TestJson>) -> String {
    format!("Hello {}\n", user.name)
}

// 파일 업로드
// #[derive(Deserialize, ToSchema)]
// struct UploadForm {
//     #[schema(format = Binary)]
//     file: String,
// }
#[utoipa::path(
    post,
    path = "/file",
    tag = HELLO_TAG,
    request_body(
        content = Vec<u8>,
        content_type = mime::MULTIPART_FORM_DATA.as_ref(),
        description = "get File",
    ),
    responses(
        (
            status = 200,
            description = "get file, and name : length print",
            body = String,
            example = "file_name : 1002"
        )
    )
)]
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

#[utoipa::path(
    get,
    path = "/header1",
    tag = HELLO_TAG,
    description = "Use HeaderMap",
    params(
        ("User-Agent" = String, Header, description = "User agent header"),
        ("Content-Type" = String, Header, description = "Content type header"),
    ),
    responses(
        (
            status = 200,
            description = "get file, and name : length print",
            body = String,
            example = "User-Agent: , Content-Type: "
        )
    )
)]
#[debug_handler]
async fn header_hello1(headers: HeaderMap) -> String {
    let user_agent = headers
        .get(USER_AGENT)
        .map(|v| v.to_owned())
        .unwrap_or(HeaderValue::from_name(USER_AGENT));

    let content_type = headers
        .get(CONTENT_TYPE)
        .map(|v| v.to_owned())
        .unwrap_or(HeaderValue::from_name(CONTENT_TYPE));

    format!(
        "User-Agent: {:?}, Content-Type: {:?}\n",
        user_agent, content_type
    )
}

// 이거 말고도 axum-extra crate의 TypeHeader를 이용할 수도 있는데,
// 특정 헤더를 각각 가져올 수 있는 기능이 있다
// 이렇게 하면 특정 헤더가 누락되면 에러가 발생한다

#[utoipa::path(
    get,
    path = "/header2",
    tag = HELLO_TAG,
    description = "Use TypeHeader (axum-extra)",
    params(
        ("User-Agent" = String, Header, description = "User agent header"),
        ("Content-Type" = String, Header, description = "Content type header"),
    ),
    responses(
        (
            status = 200,
            description = "get file, and name : length print",
            body = String,
            example = "User-Agent: , Content-Type: "
        )
    )
)]
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
#[derive(Serialize, ToSchema)]
struct JsonTest {
    #[schema(example = "Hello Json!")]
    message: String,
}
#[utoipa::path(
    get,
    path = "/json_response",
    tag = HELLO_TAG,
    description = "Json Response Test",
    responses(
        (
            status = 200,
            description = "json",
            body = JsonTest,
        )
    )
)]
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

#[utoipa::path(
    get,
    path = "/status_code",
    tag = HELLO_TAG,
    description = "Status Code Test",
    responses(
        (
            status = StatusCode::CREATED,
            description = "StatusCode is Created",
            body = String,
            example = "Hello StatusCode"
        )
    )
)]
#[debug_handler]
// async fn response_status_code() -> StatusCode {
async fn response_status_code() -> (StatusCode, String) {
    (StatusCode::CREATED, "Hello StatusCode!\n".to_string())
    // StatusCode::CREATED
    // 기타 다른 문제가 발생할시 다른형식으로 응답을 보낼 수 있다
    // (StatusCode::INTERNAL_SERVER_ERROR, "Error!!\n".to_string())
}

// RestAPI 에선 주로 해더와 상태코드, 데이터를 함께 보내므로, 다음과 같이한다

#[utoipa::path(
    get,
    path = "/rest_api",
    tag = HELLO_TAG,
    description = "Rest API Test",
    responses(
        (
            status = StatusCode::CREATED,
            description = "ContentType is text ,StatusCode is Created",
            body = String,
            example = "Hello StatusCode"
        )
    )
)]
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
#[utoipa::path(
    get,
    path = "/state_count",
    tag = HELLO_TAG,
    description = "State Counter",
    responses(
        (
            status = 200,
            description = "Counter",
            body = String,
            example = "Hello 1Times Again!\n"
        )
    )
)]
#[debug_handler]
async fn state_base_counter(State(data): State<Arc<Mutex<Vec<i32>>>>) -> String {
    let mut data = data.lock().await;
    data[0] += 1;

    format!("Hello {}Times Again!\n", data[0])
}

#[utoipa::path(
    get,
    path = "/state_app_name",
    tag = HELLO_TAG,
    description = "State App Data (not Saved)",
    responses(
        (
            status = 200,
            description = "AppState",
            body = String,
            example = "auth_token\n"
        )
    )
)]
#[debug_handler]
async fn state_appdata_name(State(data): State<HelloAppState>) -> String {
    format!("{}\n", data.auth_token)
}

#[utoipa::path(
    get,
    path = "/state_app_users",
    tag = HELLO_TAG,
    description = "State App Data (not Saved)",
    responses(
        (
            status = 200,
            description = "AppState",
            body = String,
            example = "3\n"
        )
    )
)]
#[debug_handler]
async fn state_appdata_users(State(data): State<HelloAppState>) -> String {
    format!("{}\n", data.current_users)
}

// extension도 값은 무조건 Clone으로 가져와서 값 변경 안됨
#[utoipa::path(
    get,
    path = "/extention_users",
    tag = HELLO_TAG,
    description = "Extention App Data (not Saved)",
    responses(
        (
            status = 200,
            description = "AppState",
            body = String,
            example = "4\n"
        )
    )
)]
#[debug_handler]
async fn extension_appdata_users(Extension(mut data): Extension<HelloAppState>) -> String {
    data.current_users += 1;
    format!("{}\n", data.current_users)
}

// 프록시 예제
#[derive(Deserialize, ToSchema)]
struct Data {
    #[schema(example = "chihuahua")]
    breed: String,
    #[schema(example = 3)]
    num_pics: Option<i32>,
}

#[utoipa::path(
    post,
    path = "/proxy",
    tag = HELLO_TAG,
    description = "Test Proxy and State Chache",
    request_body(
        content = Data,
        content_type = mime::APPLICATION_JSON.as_ref(),
        description = "Get breed and (optional)num_pics"
    ),
    responses(
        (
            status = 200,
            description = "ProxyData",
        )
    )
)]
#[debug_handler(state = HelloState)]
async fn hello_proxy(
    State(state): State<Arc<Mutex<HashMap<String, (Bytes, usize)>>>>,
    State(client): State<Client>,
    Json(data): Json<Data>,
) -> Result<(StatusCode, Bytes), AppError> {
    let need_pics;
    if let Some(body) = state.lock().await.get(&data.breed) {
        need_pics = if let Some(num_pics) = data.num_pics {
            num_pics
        } else {
            1
        };

        if need_pics == body.1 as i32 {
            return Ok((StatusCode::OK, body.0.clone()));
        }
    }

    let mut url = format!("https://dog.ceo/api/breed/{}/images/random", &data.breed);

    if let Some(num_pics) = data.num_pics {
        url.push_str(format!("/{}", num_pics).as_str());
    }

    let res = client.get(url).send().await?;

    let code = res.status().as_u16();
    let body = res.bytes().await?;

    state.lock().await.insert(
        data.breed,
        (body.clone(), data.num_pics.unwrap_or(1) as usize),
    );

    // 여기서 unwrap은 동작할것이 자명하기 때문에 그냥 넘어가자
    Ok((StatusCode::from_u16(code).unwrap(), body))
}

#[derive(Deserialize, IntoParams)]
struct HelloUserCondition {
    id: Option<i32>,
    like_user: Option<String>,
    like_pass: Option<String>,
    gt_id: Option<i32>,
    lt_id: Option<i32>,
    limit: Option<u64>,
}
#[utoipa::path(
    get,
    path = "/db/select",
    tag = HELLO_TAG,
    params(HelloUserCondition),
    responses(
        (
            status = 200,
            body = UserDTO,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            body = String,
            example = "Somthing is wrong about Database",
        ),
    ),
    security(
        (), // api_key가 필요 없을때 활용
        ("api_key" = [])
    )
)]
#[debug_handler]
async fn hello_user_select(
    Query(opt): Query<HelloUserCondition>,
    State(pool): State<DatabaseConnection>,
    headers: HeaderMap,
) -> Result<Json<Vec<UserDTO>>, AppError> {
    match check_api_key(false, headers) {
        Ok(_) => {}
        Err(err) => return Err(err),
    }
    if let Some(id) = opt.id {
        return Ok(Json(vec![hello_select_one(&pool, id).await?.into()]));
    }

    let mut condition = Condition::all();
    if let Some(like_user) = opt.like_user {
        condition = condition.add(users::Column::Username.like(format!("%{}%", like_user)));
    }
    if let Some(like_pass) = opt.like_pass {
        condition = condition.add(users::Column::Password.like(format!("%{}%", like_pass)));
    }
    if let Some(gt_id) = opt.gt_id {
        if let Some(lt_id) = opt.lt_id {
            let temp = Condition::all()
                .add(users::Column::Id.gt(gt_id))
                .add(users::Column::Id.lt(lt_id));
            condition = condition.add(temp);
        } else {
            condition = condition.add(users::Column::Id.gt(gt_id));
        }
    } else if let Some(lt_id) = opt.lt_id {
        condition = condition.add(users::Column::Id.lt(lt_id));
    }

    let result = hello_select_all(&pool, condition, opt.limit).await?;
    Ok(Json(result.into_iter().map(|v| v.into()).collect()))
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
enum HelloUserExec {
    One1,
    One2,
    Many,
}

#[derive(Deserialize, IntoParams, ToSchema)]
struct HelloUserExecCommand {
    command: HelloUserExec,
    username: String,
    password: String,
}
#[utoipa::path(
    get,
    path = "/db/insert",
    tag = HELLO_TAG,
    params(HelloUserExecCommand),
    responses(
        (status = StatusCode::CREATED),
        (status = StatusCode::INTERNAL_SERVER_ERROR),
        (status = StatusCode::BAD_REQUEST),
    ),
    security(
        ("api_key" = [])
    )
)]
#[debug_handler]
async fn hello_user_insert(
    Query(command): Query<HelloUserExecCommand>,
    State(pool): State<DatabaseConnection>,
    headers: HeaderMap,
) -> Result<StatusCode, AppError> {
    match check_api_key(true, headers) {
        Ok(_) => {}
        Err(err) => return Err(err),
    }
    match command.command {
        HelloUserExec::One1 => {
            hello_insert_one1(&pool, command.username, command.password).await?;
        }
        HelloUserExec::One2 => {
            hello_insert_one2(&pool, command.username, command.password).await?;
        }
        _ => {
            return Err(AppError::new(
                StatusCode::BAD_REQUEST,
                "Many Insert is moved other url\nPlease use the /hello/db/insert_many",
            ));
        }
    }
    Ok(StatusCode::CREATED)
}

#[derive(Deserialize, ToSchema)]
struct HelloUserInsertManyCommand {
    username: String,
    password: String,
}

#[utoipa::path(
    get,
    path = "/db/insert_many",
    tag = HELLO_TAG,
    request_body(
        content = Vec<HelloUserInsertManyCommand>,
        content_type = mime::APPLICATION_JSON.as_ref(),
        description = "User List"
    ),
    responses(
        (status = StatusCode::CREATED),
        (status = StatusCode::INTERNAL_SERVER_ERROR),
        (status = StatusCode::BAD_REQUEST),
    ),
    security(
        ("api_key" = [])
    )
)]
#[debug_handler]
async fn hello_user_insert_many(
    State(pool): State<DatabaseConnection>,
    headers: HeaderMap,
    Json(command): Json<Vec<HelloUserInsertManyCommand>>,
) -> Result<StatusCode, AppError> {
    match check_api_key(true, headers) {
        Ok(_) => {}
        Err(err) => return Err(err),
    }
    let models: Vec<(String, String)> = command
        .into_iter()
        .map(|com| (com.username, com.password))
        .collect();

    hello_insert_many(&pool, models).await?;
    Ok(StatusCode::CREATED)
}

#[derive(Deserialize, ToSchema)]
struct HelloUserUpdateCommand {
    model: Option<HelloUserDeleteCommand>,
    change_model: UserDTO,
}
#[utoipa::path(
    post,
    path = "/db/update/{exec}",
    params(
        ("exec" = HelloUserExec, Path, description = "Execution type")
    ),
    request_body(
        content = HelloUserUpdateCommand,
        content_type = mime::APPLICATION_JSON.as_ref(),
        description = "find user info mation"
    ),
    responses(
        (status = StatusCode::OK),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    ),
    security(
        ("api_key" = [])
    )
)]
#[debug_handler]
async fn hello_user_update(
    Path(exec): Path<HelloUserExec>,
    State(pool): State<DatabaseConnection>,
    headers: HeaderMap,
    Json(model): Json<HelloUserUpdateCommand>,
) -> Result<StatusCode, AppError> {
    match check_api_key(true, headers) {
        Ok(_) => {}
        Err(err) => return Err(err),
    }
    match exec {
        HelloUserExec::One1 => {
            if let Some(find) = model.model {
                if let Some(id) = find.id {
                    hello_update_one1(
                        &pool,
                        id,
                        model.change_model.username,
                        model.change_model.password,
                    )
                    .await?;
                }
            }
        }
        HelloUserExec::One2 => {
            if let Some(find) = model.model {
                if let (Some(id), Some(username), Some(password)) =
                    (find.id, find.username, find.password)
                {
                    let find = users::Model {
                        id,
                        username,
                        password,
                    };
                    hello_update_one2(
                        &pool,
                        find,
                        model.change_model.username,
                        model.change_model.password,
                    )
                    .await?;
                }
            }
        }
        HelloUserExec::Many => {
            hello_update_many(
                &pool,
                model.change_model.username,
                model.change_model.password,
            )
            .await?;
        }
    }
    Ok(StatusCode::OK)
}

#[derive(Deserialize, ToSchema)]
struct HelloUserDeleteCommand {
    id: Option<i32>,
    username: Option<String>,
    password: Option<String>,
}
#[utoipa::path(
    post,
    path = "/db/delete/{exec}",
    params(
        ("exec" = HelloUserExec, Path, description = "Execution type")
    ),
    request_body(
        content = HelloUserDeleteCommand,
        content_type = mime::APPLICATION_JSON.as_ref(),
        description = "find user info mation"
    ),
    responses(
        (status = StatusCode::OK),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    ),
    security(
        ("api_key" = [])
    )
)]
#[debug_handler]
async fn hello_user_delete(
    Path(command): Path<HelloUserExec>,
    Query(model): Query<HelloUserDeleteCommand>,
    State(pool): State<DatabaseConnection>,
    headers: HeaderMap,
) -> Result<StatusCode, AppError> {
    match check_api_key(true, headers) {
        Ok(_) => {}
        Err(err) => return Err(err),
    }
    match command {
        HelloUserExec::One1 => {
            if let Some(id) = model.id {
                hello_delete_by_id(&pool, id).await?;
            } else {
                return Err(AppError::new(
                    StatusCode::BAD_REQUEST,
                    "Need input the user info",
                ));
            }
        }
        HelloUserExec::One2 => {
            if let (Some(username), Some(password)) = (model.username, model.password) {
                hello_delete_by_model(
                    &pool,
                    users::Model {
                        id: -1,
                        username,
                        password,
                    },
                )
                .await?;
            } else {
                return Err(AppError::new(
                    StatusCode::BAD_REQUEST,
                    "Need input the user info",
                ));
            }
        }
        HelloUserExec::Many => {
            hello_delete_many(&pool).await?;
        }
    }

    Ok(StatusCode::OK)
}

// API Key가 유효한지 확인하는 함수
// GPT는 미들웨어로 분리하는걸 권장함
fn check_api_key(require_api_key: bool, headers: HeaderMap) -> Result<(), AppError> {
    match headers.get("hello_apikey") {
        // 근데 이거 이렇게 하면 "utoipa-rocks"로 api_key가 고정되는거 아닌가
        Some(header) if header != "utoipa-rocks" => {
            Err(AppError::new(StatusCode::UNAUTHORIZED, "incorrect api key"))
        }
        None if require_api_key => Err(AppError::new(StatusCode::UNAUTHORIZED, "missing api key")),
        _ => Ok(()),
    }
}

pub fn hello_router(pool: DatabaseConnection) -> Router {
    let db_router = OpenApiRouter::new()
        .routes(routes!(hello_user_select))
        .routes(routes!(hello_user_insert))
        .routes(routes!(hello_user_insert_many))
        .routes(routes!(hello_user_update))
        .routes(routes!(hello_user_delete));

    let hello_router = OpenApiRouter::new()
        .routes(routes!(param_query_with_hashmap))
        .routes(routes!(path_query))
        .routes(routes!(param_query_with_struct))
        .routes(routes!(body_query_text))
        .routes(routes!(body_query_bytes))
        .routes(routes!(body_query_form))
        .routes(routes!(body_query_json))
        .routes(routes!(body_query_file_upload))
        .routes(routes!(header_hello1))
        .routes(routes!(header_hello2))
        .routes(routes!(response_json))
        .routes(routes!(response_status_code))
        .routes(routes!(response_base_rest_api))
        // 라우터에서 상태에 대한 정보를 넘기는 방법은 다음과 같다
        // 상태 관리를 위해서 가져오는 데이터
        // 물론 각 요청별로 상태를 다르게 하기 위해서 get(함수명).with_state() 도 가능하다
        // route뒤의 with_state는 마지막 with_state에 따라서 그 위의 route된 모든 함수에 대응한다
        .routes(routes!(state_base_counter))
        // .with_state(get_base_state())
        .routes(routes!(state_appdata_name))
        .routes(routes!(state_appdata_users))
        // .with_state(get_hello_app_state())
        // 권장하지 않는 방식의 상태관리법 (구버전)
        .routes(routes!(extension_appdata_users))
        .layer(Extension(get_hello_app_state()))
        // 프록시 서버
        .routes(routes!(hello_proxy))
        // .with_state(get_proxy_state());
        .nest("/db", db_router)
        .with_state(get_hello_state(pool));

    // 상태관리의 경우 마지막 상태가 확정된 상태이므로,
    // 가능하면 서브트리당 하나만 쓰거나 부모 트리를 상속받아야한다
    set_router(hello_router)
}
