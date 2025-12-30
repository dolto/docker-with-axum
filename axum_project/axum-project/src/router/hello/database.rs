use crate::entities::*;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{self, NotSet, Set},
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, InsertResult, IntoActiveModel,
    QueryFilter, QueryOrder, QuerySelect, UpdateResult,
    prelude::Expr,
};

// DbErr에 대해서
// pub async fn hello_db_err(conn: &DatabaseConnection) {
//     let res = hello_insert_one1(conn).await;

//     match res {
//         Ok(user) => {
//             println!("{:?}", user);
//         }
//         // 공부용이라 sql_err와 exec(runtimeerr)를 둘 다 이용했지만
//         // 에러를 바라보는 관점에 따라 하나만 만들어야한다
//         Err(err) => {
//             // sql_err를 통해 가져온 에러로, 의미있는 DB제약 위반을 골라서 처리
//             // 주로 비즈니스 로직에대한 판단
//             match err.sql_err() {
//                 Some(err) => match err {
//                     SqlErr::UniqueConstraintViolation(detail) => {
//                         println!("{}", detail);
//                     }
//                     SqlErr::ForeignKeyConstraintViolation(detail) => {
//                         println!("{}", detail);
//                     }
//                     _ => {
//                         println!("{:?}", err);
//                     }
//                 },
//                 None => {}
//             }

//             // 시스템/인프라에 대한 판단 (에러 추적)
//             match err {
//                 // SeaORM에서 생성된 에러를 처리
//                 DbErr::Exec(RuntimeErr::Internal(details)) => {
//                     println!("{}", details);
//                 }
//                 // 쿼리 수행중 발생하는 에러를 처리
//                 DbErr::Exec(RuntimeErr::SqlxError(error)) => match error {
//                     sqlx::Error::Database(db_error) => {
//                         if let Some(code) = db_error.code() {
//                             match code.as_ref() {
//                                 "23505" => println!("Unique constraint violation"),
//                                 "23503" => println!("Foreign key constraint violation"),
//                                 _ => println!("Other database error: {}", code),
//                             }
//                         }
//                     }
//                     _ => println!("Other SQLx error: {:?}", error),
//                 },
//                 _ => {}
//             }
//         }
//     }
// }

// Entity는 DB의 레코드를 나타냄
pub async fn hello_select_one(conn: &DatabaseConnection) -> users::Model {
    let user = users::Entity::find()
        .filter(users::Column::Username.eq("dolto"))
        .one(conn)
        .await
        .unwrap()
        .unwrap();

    user
}

pub async fn hello_select_all(conn: &DatabaseConnection) -> Vec<users::Model> {
    let user = users::Entity::find()
        .filter(users::Column::Id.gt(1))
        .order_by_asc(users::Column::Username)
        .all(conn)
        .await
        .unwrap();

    user
}

pub async fn hello_select_limit(conn: &DatabaseConnection, limit: u64) -> Vec<users::Model> {
    let user = users::Entity::find()
        .filter(users::Column::Id.gt(1))
        .order_by_asc(users::Column::Username)
        .limit(limit)
        .all(conn)
        .await
        .unwrap();

    user
}

// ActiveModel은 DB의 레코드를 삽입/수정 가능하게끔 해줌
pub async fn hello_insert_one1(conn: &DatabaseConnection) -> Result<users::Model, DbErr> {
    let new_user = users::ActiveModel {
        id: NotSet,
        username: Set("dolto".to_owned()),
        password: Set("1234".to_owned()),
    }
    .insert(conn)
    .await?;

    Ok(new_user)
}

pub async fn hello_insert_one2(
    conn: &DatabaseConnection,
) -> Result<InsertResult<users::ActiveModel>, DbErr> {
    let new_user = users::ActiveModel {
        id: NotSet,
        username: Set("Tomy".to_owned()),
        password: Set("1234".to_owned()),
    };
    let result = users::Entity::insert(new_user).exec(conn).await?;

    Ok(result)
}

pub async fn hello_insert_many(
    conn: &DatabaseConnection,
) -> Result<InsertResult<users::ActiveModel>, DbErr> {
    let new_users = vec![
        users::ActiveModel {
            id: NotSet,
            username: Set("Jim".to_owned()),
            password: Set("4321".to_owned()),
        },
        users::ActiveModel {
            id: NotSet,
            username: Set("Kidy".to_owned()),
            password: Set("4321".to_owned()),
        },
        users::ActiveModel {
            id: NotSet,
            username: Set("Wellny".to_owned()),
            password: Set("4321".to_owned()),
        },
        users::ActiveModel {
            id: NotSet,
            username: Set("Jerry".to_owned()),
            password: Set("4321".to_owned()),
        },
    ];

    let result = users::Entity::insert_many(new_users).exec(conn).await?;

    // 마지막으로 추가된 레코드의 id값을 구할 수 있다
    // 자동증가를 설정한 경우에 한해서
    let last_id = result.last_insert_id;
    println!("last id is {}", last_id);

    Ok(result)
}

pub async fn hello_update_one(conn: &DatabaseConnection) -> Result<users::Model, DbErr> {
    // 수정할 데이터를 가져와서 into()로 ActiveModel로 변환
    let mut user = users::Entity::find()
        .filter(users::Column::Username.eq("dolto"))
        .one(conn)
        .await?
        .unwrap()
        .into_active_model();

    // 데이터를 수정하고
    user.username = ActiveValue::Set("Dolto".to_owned());
    user.password = ActiveValue::Set("4321".to_owned());

    // 그것을 반영할 수 있다
    let updated_user = user.update(conn).await?;

    Ok(updated_user)
}

pub async fn hello_update_many(conn: &DatabaseConnection) -> Result<UpdateResult, DbErr> {
    // 일괄 수정은 다음과 같이 할 수 있다
    let updated_user = users::Entity::update_many()
        .col_expr(users::Column::Password, Expr::value("Hello Password"))
        // 이런식으로 필터도 넣을 수 있다
        // .filter(users::Column::Password.eq("test"))
        .exec(conn)
        .await?;

    Ok(updated_user)
}

// userid 1 삭제
pub async fn hello_delete_by_model(
    conn: &DatabaseConnection,
) -> Result<sea_orm::DeleteResult, DbErr> {
    let user = users::Entity::find_by_id(1)
        .one(conn)
        .await?
        .unwrap()
        .into_active_model();
    let delete_user = users::Entity::delete(user).exec(conn).await?;

    Ok(delete_user)
}

pub async fn hello_delete_by_id(conn: &DatabaseConnection) -> Result<sea_orm::DeleteResult, DbErr> {
    // userId 2 삭제
    let delete_user = users::Entity::delete_by_id(2).exec(conn).await?;

    Ok(delete_user)
}

pub async fn hello_delete_many(conn: &DatabaseConnection) -> Result<sea_orm::DeleteResult, DbErr> {
    // user 전체 삭제
    let delete_users = users::Entity::delete_many().exec(conn).await?;

    Ok(delete_users)
}
