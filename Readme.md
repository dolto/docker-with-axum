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
