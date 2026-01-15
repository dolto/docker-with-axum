use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::time::Duration;
use tracing::log::LevelFilter;

pub const DB_ERR_MESSAGE: &str = "Somthing is wrong about Database";
pub async fn init_db() -> Result<DatabaseConnection, DbErr> {
    // 컨테이너 내부에 이미 환경변수 설정이 되어있음
    // 만약 파일에서 가져오고 싶다면 dotenv 크레이트로 가져오면 된다
    let database_url = std::env::var("DATABASE_URL").unwrap();
    // let conn = Database::connect(&database_url).await.unwrap();

    let mut opt = ConnectOptions::new(database_url);

    // 커넥션풀 최대 연결 수 설정
    opt.max_connections(100)
        // 커넥션풀 최소 연결 수 설정
        .min_connections(5)
        // 연결 시도 타임아웃 설정
        .connect_timeout(Duration::from_secs(8))
        // 유휴상태의 연결 획득 시도 타임아웃 설정
        .acquire_timeout(Duration::from_secs(8))
        // 유휴상태의 연결 타임아웃 설정
        .idle_timeout(Duration::from_secs(8))
        // 연결 수명 타임아웃 설정
        .max_lifetime(Duration::from_secs(8))
        // SQLx로깅 활성화
        .sqlx_logging(true)
        // SQLx로그 레벨 설정
        .sqlx_logging_level(LevelFilter::Info)
        .set_schema_search_path("public");
    // Postgres DB이름이 axum이지 스키마가 아니다. 보통 기본 스키마는 public이다
    // .set_schema_search_path("axum");

    Ok(Database::connect(opt).await?)
}
