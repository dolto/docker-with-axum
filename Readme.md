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
│   ├── Cargo.lock
│   ├── Cargo.toml
│   ├── Dockerfile.client
│   ├── Dockerfile.fullstack
│   ├── backend
│   ├── client
│   │   ├── AGENTS.md
│   │   ├── Cargo.toml
│   │   ├── Dioxus.toml
│   │   ├── README.md
│   │   └── src
│   │       ├── component
│   │       │   ├── login.rs
│   │       │   └── mod.rs
│   │       ├── main.rs
│   │       └── util.rs
│   ├── fullstack
│   │   ├── Cargo.lock
│   │   ├── Cargo.toml
│   │   ├── Dioxus.toml
│   │   ├── res
│   │   │   └── tacit-css-1.9.5.min.css
│   │   ├── src
│   │   │   ├── database.rs
│   │   │   ├── front
│   │   │   │   ├── mod.rs
│   │   │   │   ├── page
│   │   │   │   │   ├── component
│   │   │   │   │   │   ├── error_layout.rs
│   │   │   │   │   │   ├── login.rs
│   │   │   │   │   │   └── mod.rs
│   │   │   │   │   ├── home.rs
│   │   │   │   │   └── mod.rs
│   │   │   │   └── util.rs
│   │   │   ├── main.rs
│   │   │   ├── middle.rs
│   │   │   ├── open_api.rs
│   │   │   ├── resources
│   │   │   │   ├── dto
│   │   │   │   │   ├── fullstack_extension.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── user.rs
│   │   │   │   ├── entities
│   │   │   │   │   ├── category.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── prelude.rs
│   │   │   │   │   ├── product.rs
│   │   │   │   │   ├── refresh_token.rs
│   │   │   │   │   └── users.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── style
│   │   │   │       └── mod.rs
│   │   │   ├── router
│   │   │   │   ├── api
│   │   │   │   │   ├── auth.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── user.rs
│   │   │   │   ├── hello
│   │   │   │   │   ├── database.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── open_api.rs
│   │   │   │   │   └── state.rs
│   │   │   │   └── mod.rs
│   │   │   ├── utils
│   │   │   │   ├── errors.rs
│   │   │   │   ├── hash.rs
│   │   │   │   ├── jwt.rs
│   │   │   │   └── mod.rs
│   │   │   └── ws
│   │   │       ├── chat.rs
│   │   │       ├── mod.rs
│   │   │       └── state.rs
│   │   └── state-fullstack-context-macro
│   │       ├── Cargo.toml
│   │       └── src
│   │           └── lib.rs
│   ├── shared
│   │   ├── Cargo.toml
│   │   └── res
│   │       └── tacit-css-1.9.5.min.css
│   └── target_docker
├── db
│   ├── Dockerfile
│   └── migration
│       ├── Cargo.lock
│       ├── Cargo.toml
│       ├── README.md
│       └── src
│           ├── lib.rs
│           ├── m20251228_110826_create_table.rs
│           ├── m20260109_003305_update.rs
│           ├── m20260119_020622_update.rs
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
    ├── DB_URL.env
    └── DEV.env
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
```DEV.env
RUST_LOG=debug
SECRET_KEY=1234
CARGO_TARGET_DIR=/home/app/target_docker
GOOGLE_CLIENT_ID=구글 클라이언트 아이디
GOOGLE_CLIENT_SECRET_KEY=구글 클라이언트 비밀키
LOGIN_REDIRECT=구글 클라이언트 리디렉션 주소
```
3. docker compose 환경변수 설정 (.env)
- 터미널에서 현재 유저의 uid와 gid를 구한다 (유저 아이디, 그룹 아이디)
```sh
  id -u
  id -g
```
- 두 아이디번호를 다음과 같이 입력해준다
```.env
CURRENT_USER=1000:1000
```
- 마이그레이션, upper등 파일을 생성해주는 컨테이너가 root가 아닌 사용자 권한으로 생성해주기 때문에, 권한 에러를 방지할 수 있다
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

#### open api 테스트
- <img width="844" height="558" alt="image" src="https://github.com/user-attachments/assets/0e9c46cc-4045-41de-967d-fd7813840dea" />
- <img width="909" height="400" alt="image" src="https://github.com/user-attachments/assets/7c2b2783-cfc1-41b4-96b1-5ff168a1dee7" />
##### 도메인 목록
- OpenAi 문서 링크
- /도메인/doc/scalar (or redoc)
- api/auth
- api/user
- ws
- 예외적으로 hello는 hello/scalar에 위치해있다

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
  # 마이그레이션 생성 (이름은 update)
  docker compose run --rm sea-orm-cli
```
