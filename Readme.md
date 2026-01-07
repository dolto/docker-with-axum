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
- Postgresql

#### Management
- Git (Obviously)

#### 프로젝트 구조
```sh
.
├── Readme.md
├── axum_project
│   ├── Dockerfile
│   └── axum-project
│       ├── Cargo.lock
│       ├── Cargo.toml
│       └── src
│           ├── database.rs
│           ├── entities
│           │   ├── category.rs
│           │   ├── mod.rs
│           │   ├── prelude.rs
│           │   ├── product.rs
│           │   └── users.rs
│           ├── lib.rs
│           ├── main.rs
│           ├── middle.rs
│           ├── open_api.rs
│           ├── router
│           │   ├── api
│           │   │   ├── auth.rs
│           │   │   ├── mod.rs
│           │   │   └── user.rs
│           │   ├── hello
│           │   │   ├── database.rs
│           │   │   ├── mod.rs
│           │   │   ├── open_api.rs
│           │   │   └── state.rs
│           │   └── mod.rs
│           ├── utils
│           │   ├── errors.rs
│           │   ├── hash.rs
│           │   ├── jwt.rs
│           │   └── mod.rs
│           └── ws
│               └── mod.rs
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
├── docker-compose.yaml
└── env_files
    ├── DB_ADMIN.env
    ├── DB_URL.env
    └── DEV.env
```

### 프로젝트 초기 설정(초본)
1. 환경변수 파일을 생성
```sh
└── env_files
    ├── DB_ADMIN.env
    └── DB_URL.env
```
2. 환경변수 설정(예시)
```DB_URL.env
DATABASE_URL=postgres://axum:1234@db:5432/axum
```
```DB_ADMIN.env
POSTGRES_USER=dolto
POSTGRES_DB=dolto
POSTGRES_PASSWORD=dolto
```
3. ```docker compose --file docker-compose.yaml up --detach```
4. 생성된 컨테이너 두개를 확인
```sh
  docker container ls
```
5. DB로 들어가 컨테이너 설정(예제를 따랐다면 다음과 같이 설정)
- ```docker container exec -it axum-postgres psql -U dolto```
```sql
  create user axum;
  alter user axum password '1234';
  create database axum;
  alter database axum owner to axum;
  \q
```
6. 여담
- ADMIN과 DB_URL의 유저를 같은 사람으로 설정했다면 데이터베이스만 생성하면 된다

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
- OpenAi 문서 링크
- /api/user/doc/scalar (or redoc)
- /hello/scalar (or redoc)
- 밑의 curl도 가능하지만, scalar나 redoc으로 api를 확인할 밑 테스트 할 수 있다
- <img width="844" height="558" alt="image" src="https://github.com/user-attachments/assets/0e9c46cc-4045-41de-967d-fd7813840dea" />
- <img width="909" height="400" alt="image" src="https://github.com/user-attachments/assets/7c2b2783-cfc1-41b4-96b1-5ff168a1dee7" />
```sh
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

  # Hello User Database
  # Insert
  curl -i "http://localhost:8080/hello/db/insert?command=one1&username=user&password=pass1" -H 'hello_apikey:utoipa-rocks'
  curl -i "http://localhost:8080/hello/db/insert?command=one2&username=user&password=pass2" -H 'hello_apikey:utoipa-rocks'
  curl -i -X POST "http://localhost:8080/hello/db/insert_many" \
  -H "Content-Type: application/json" \
  -H 'hello_apikey:utoipa-rocks' \
  -d '[
    {"username":"erin","password":"pass5"},
    {"username":"frank","password":"pass6"},
    {"username":"gina","password":"pass7"}
  ]'

  # Select
  curl -X GET "http://localhost:8080/hello/db/select" -i
  curl -X GET "http://localhost:8080/hello/db/select?id=1" -i
  # 섞어 쓸 수도 있음
  curl -X GET "http://localhost:8080/hello/db/select?like_user=dolto" -i
  curl -X GET "http://localhost:8080/hello/db/select?like_pass=1234" -i
  curl -i "http://localhost:8080/hello/db/select?gt_id=1&lt_id=10"
  curl -i "http://localhost:8080/hello/db/select?limit=2"

  # Update (모델을 가져오든 id로 가져오든 결국 pk를 기준으로 변경하는듯)
  curl -i -X POST "http://localhost:8080/hello/db/update/one1" \
  -H "Content-Type: application/json" \
  -H 'hello_apikey:utoipa-rocks'
  -d '{
    "model":  { "id": 23, "username": "alice", "password": "pass1" },
    "change_model": { "id": 23, "username": "alice_updated", "password": "pass1" }
  }'
  curl -i -X POST "http://localhost:8080/hello/db/update/one2" \
  -H "Content-Type: application/json" \
  -H 'hello_apikey:utoipa-rocks'
  -d '{
    "model":  { "id": 23, "username": "alice", "password": "pass1" },
    "change_model": { "id": 23, "username": "alice_updated", "password": "pass1" }
  }'
  curl -i -X POST "http://localhost:8080/hello/db/update/many" \
  -H "Content-Type: application/json" \
  -H 'hello_apikey:utoipa-rocks'
  -d '{
    "model": { "id": 0, "username": "dummy", "password": "dummy" },
    "change_model": { "id": 0, "username": "GLOBAL_UPDATED", "password": "global_pw" }
  }'

  # Delete
  curl -i "http://localhost:8080/hello/db/delete/one1?id=1" -H 'hello_apikey:utoipa-rocks'
  curl -i "http://localhost:8080/hello/db/delete/one2?username=name&password=pass2" -H 'hello_apikey:utoipa-rocks'
  curl -i "http://localhost:8080/hello/db/delete/many" -H 'hello_apikey:utoipa-rocks'

  # 유저 정보 가져오기
  curl 'http://localhost:8080/api/user/get?id=null&username=null&password=null'

  # 유저 회원가입
  curl http://localhost:8080/api/user/post \
  --request POST \
  --header 'Content-Type: application/json' \
  --data '{
    "id": null,
    "password": null,
    "username": null
  }'

  # 유저 정보 변경
  curl http://localhost:8080/api/user/put \
  --request PUT \
  --header 'Content-Type: application/json' \
  --data '{
    "id": null,
    "password": null,
    "username": null
  }'

  # 유저 삭제 (id만 받아도 됨)
  curl http://localhost:8080/api/user/delete \
  --request DELETE \
  --header 'Content-Type: application/json' \
  --data '{
    "id": null,
    "password": null,
    "username": null
  }'

  # Jwt 토큰 발급
  curl -X POST "http://localhost:8080/api/auth/login" -H "Content-Type: application/json" -d '{ "username":"dolto", "password":"1234" } '

  # Jwt 토큰 사용
  curl -X GET "http://localhost:8080/" \
    -H "token"
```
- 당연하지만 나중엔 회원가입을 제외한 모든 user관련 api를 jwt토큰을 통해 인증/인가를 통해서 동작하게 할 것이다

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
