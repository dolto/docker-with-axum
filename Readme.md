# 실전 백엔드 러스트 Axum 프로그래밍
### ...을 입맛대로 수정해서 공부하는 레포

## 환경

#### OS
- Window11 with WSL

#### Editor
- Helix

#### Language and Framework
- Rust
- Docker (Docker compose V2)
- Axum
- SeaORM

#### Management
- Git (Obviously)

#### 프로젝트 구조
```sh
├── Readme.md
├── axum_project
│   ├── Dockerfile
│   └── axum-project
│       ├── Cargo.lock
│       ├── Cargo.toml
│       └── src
│           ├── entities
│           │   ├── category.rs
│           │   ├── mod.rs
│           │   ├── prelude.rs
│           │   ├── product.rs
│           │   └── users.rs
│           ├── main.rs
│           └── router
│               ├── hello
│               │   ├── mod.rs
│               │   └── state.rs
│               ├── mod.rs
│               └── user
│                   └── mod.rs
├── db
│   ├── Dockerfile
│   └── migration
│       ├── Cargo.lock
│       ├── Cargo.toml
│       ├── README.md
│       └── src
│           ├── lib.rs
│           ├── m20251228_110826_create_table.rs
│           └── main.rs
└── docker-compose.yaml
```

### 프로젝트 초기 설정(초본)
1. ```docker compose --file docker-compose.yaml up --detach```
2. 생성된 컨테이너 두개를 확인
```sh
  docker container ls
```
3. DB로 들어가 컨테이너 설정
- ```docker container exec -it axum-postgres psql -U dolto```
```sql
  create user axum;
  alter user axum password '1234';
  create database axum;
  alter database axum owner to axum;
  \q
```

### 소스코드
- axum_project/axum-project 에서 수정 가능
- db/ 마이그레이션 코드

#### 라우터와 핸들러 공부 소스
- /axum_project/axum-project/src/router/user/mod.rs
##### URL 파라메터
- Path, Query(HashMap, Struct)
##### Body
- Text, Json, Form, FormData (file)
##### Debug 를 위한 #[debug_handler]
- 핸들러 함수에 다음 어노테이션을 걸면, 컴파일시 발생하는 에러를 더 명확하게 알 수 있다

#### 테스트 curl 정보
```sh
  # 기본 요청
  curl -X GET "http://localhost:8080/"
  curl -X POST "http://localhost:8080/"
  curl -X PUT "http://localhost:8080/"
  curl -X DELETE "http://localhost:8080/"

  # Path & Parameter
  curl -X GET "http://localhost:8080/hello/path/1/dolto"
  curl -X GET "http://localhost:8080/hello/param1?id=12&name=dolto"
  curl -X GET "http://localhost:8080/hello/param2?id=22&name=dolto"

  # Body Text & Bytes
  curl -X POST "http://localhost:8080/hello/text" -d "dolto"
  curl -X POST "http://localhost:8080/hello/bytes" -d "dolto"

  # Body Json & Form
  curl -X POST "http://localhost:8080/hello/json" -H "Content-Type: application/json" -d '{"name":"dolto"}'
  curl -X POST "http://localhost:8080/hello/form" -H "Content-Type: application/x-www-form-urlencoded" -d "name=dolto"

  # Body FormData (file & Form)
  curl -X POST "http://localhost:8080/hello/file" -F "readme.md=@Readme.md" -F "dolto=dolto"

  # Header
  curl -X GET "http://localhost:8080/hello/header" -H "Content-Type: text/plain" -d "dolto"

  # Json Response
  curl -X GET "http://localhost:8080/hello/json_response"

  # StatusCode Response
  curl -X GET "http://localhost:8080/hello/status_code" -i

  # Rest API Response (Header ,(StatusCode, Data))
  curl -X GET "http://localhost:8080/hello/rest_api" -i

  # State
  curl -X GET "http://localhost:8080/hello/state_count"
  curl -X GET "http://localhost:8080/hello/state_app_name"
  curl -X GET "http://localhost:8080/hello/state_app_users"
  curl -X GET "http://localhost:8080/hello/extension_users"

  # Hello Proxy
  curl -X POST "http://localhost:8080/hello/proxy" -H "Content-Type: application/json" -d '{"breed":"chihuahua", "num_pics":3}'
```

#### SeaORM 마이그레이션 위치
- /db/migrate

#### SeaORM 으로 생성된 데이터 모델
- /axum_project/axum-project/src/entities

##### sea-orm-cli 실행 컨테이너
- DB는 로컬과 포트연결이 되어있지 않기 때문에 같은 networks에 연결된 컨테이너를 생성한다
- 기반 이미지 Dockerfile ```/db/Dockerfile```
- 마이그레이션 up을 해야 이미지를 빌드하기 때문에 최초 마이그레이션은 해야한다
```sh
  docker compose run --rm sea-orm-upper
```
- 그 이후로는 필요에 따라 실행하면된다
```sh
  # 마이그레이션 적용 sea-orm-cli migrate up
  docker compose run --rm sea-orm-upper
  # 마이그레이션 롤백(마지막 마이그레이션) sea-orm-cli migrate down
  docker compose run --rm sea-orm-downer
  # 마이그레이션 모델 생성 sea-orm-cli generate entity -o src/entities
  docker compose run --rm sea-orm-entity
```
