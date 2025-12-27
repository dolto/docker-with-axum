use std::sync::{Arc, Mutex};

pub fn get_base_state() -> Arc<Mutex<Vec<i32>>> {
    Arc::new(Mutex::new(vec![0; 3]))
}

// 이 경우엔 값 수정이 의미가 없다
// State로 가져온 값은 무조건 Clone으로 가져오기 때문
#[derive(Clone, Debug)]
pub struct HelloAppState {
    pub auth_token: String,
    pub current_users: i32,
}
pub fn get_hello_app_state() -> HelloAppState {
    HelloAppState {
        auth_token: "auth_token".to_string(),
        current_users: 3,
    }
}

// 난 안쓸 것 같지만 mecros features에는
// 데이터 구조체의 각 필드만 따로 받는 기능이 있다
// FromRef라는 어노테이션을 사용하면 된다
// #[derive(Clone, FromRef)]
// struct Temp {
//     a: String,
//     b: i32,
// }

// 그걸 다음과 같은식으로 a 를 가져올 수 있다... (with_state를 해야한다)
// async fn temp(State(auth_token): State<String>) -> String {
//     format!("{}", auth_token)
// }

// 하지만 구조체 멤버의 타입이 중복될경우 사용할 수 없다

// 또한 0.6.0 버전 이전에 쓰던 Extension이라는 기능이 있지만, 사용하지 않길 권한다
// 라우터에 layer(Extension(state))를 명시적으로 추가하고, 사용한다
// State와 마찬가지로 Clone을 사용하면서 타입 안정성을 무시하는 방법이라 쓰지 않는게 좋다
