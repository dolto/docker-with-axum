# 실전 백엔드 러스트 Axum 프로그래밍
### ...을 입맛대로 수정해서 공부하는 레포

## 환경

#### OS
- Window11 with WSL

#### Editor
- Helix

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
4. 코드를 수정하고, 핫 리로드 되는지 테스트도 해보자

### 소스코드
- axum_project/axum-project 에서 수정 가능
- db/ 에 각종 마이그레이션용 sql을 넣는 형태로 하거나, 볼륨의 덤프를 저장할 예정

#### 라우터와 핸들러 공부 소스
- /router/user/mod.rs
##### URL 파라메터
- Path, Query(HashMap, Struct)
##### Body
- Text, Json, Form, FormData (file)

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
```
