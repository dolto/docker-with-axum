use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use crate::utils::errors::AppError;

#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
#[derive(Deserialize, Serialize, Debug)]
pub struct UserCondition {
    pub id: Option<i32>,
    pub username: Option<String>,
    pub google: Option<String>,
    pub kakao: Option<String>,
    pub github: Option<String>,
    pub naver: Option<String>,
}
impl Default for UserCondition {
    fn default() -> Self {
        UserCondition {
            id: None,
            username: None,
            google: None,
            kakao: None,
            github: None,
            naver: None,
        }
    }
}
#[cfg(feature = "server")]
impl From<UserCondition> for crate::resources::entities::users::ActiveModel {
    fn from(value: UserCondition) -> Self {
        use sea_orm::ActiveValue::{NotSet, Set};

        crate::resources::entities::users::ActiveModel {
            id: NotSet,
            username: if let Some(username) = value.username {
                Set(username)
            } else {
                NotSet
            },
            google_oauth: Set(value.google),
            kakao_oauth: Set(value.kakao),
            git_hub_oauth: Set(value.github),
            naver_oauth: Set(value.naver),
        }
    }
}
#[cfg(feature = "server")]
impl UserCondition {
    pub fn make_condition(&self) -> Result<sea_orm::Condition, AppError> {
        use crate::resources::entities::users;
        use sea_orm::ColumnTrait;
        use sea_orm::Condition;

        let condition = Condition::all();
        if let Some(id) = self.id {
            let res = condition.add(users::Column::Id.eq(id));
            return Ok(res);
        } else if let Some(oauth) = &self.google {
            let res = condition.add(users::Column::GoogleOauth.eq(oauth));
            return Ok(res);
        } else if let Some(oauth) = &self.kakao {
            let res = condition.add(users::Column::KakaoOauth.eq(oauth));
            return Ok(res);
        } else if let Some(oauth) = &self.naver {
            let res = condition.add(users::Column::NaverOauth.eq(oauth));
            return Ok(res);
        } else if let Some(oauth) = &self.naver {
            let res = condition.add(users::Column::NaverOauth.eq(oauth));
            return Ok(res);
        } else if let Some(username) = &self.username {
            let res = condition.add(users::Column::Username.like(format!("%{}%", username)));
            return Ok(res);
        }

        Err(AppError::any_t_error("There is no condition"))
    }
    pub async fn post_user(self, conn: &sea_orm::DatabaseConnection) -> Result<UserDto, AppError> {
        use sea_orm::ActiveModelTrait;

        use crate::resources::entities::users;

        let model = users::ActiveModel::from(self);

        let res = model.insert(conn).await?;
        Ok(res.into())
    }
}

#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UserDto {
    pub id: i32,
    pub username: String,
    pub google: Option<String>,
    pub kakao: Option<String>,
    pub github: Option<String>,
    pub naver: Option<String>,
}
#[cfg(feature = "server")]
impl From<crate::resources::entities::users::Model> for UserDto {
    fn from(value: crate::resources::entities::users::Model) -> Self {
        UserDto {
            id: value.id,
            username: value.username,
            google: value.google_oauth,
            kakao: value.kakao_oauth,
            github: value.git_hub_oauth,
            naver: value.naver_oauth,
        }
    }
}
#[cfg(feature = "server")]
impl From<UserDto> for crate::resources::entities::users::Model {
    fn from(value: UserDto) -> Self {
        crate::resources::entities::users::Model {
            id: value.id,
            username: value.username,
            google_oauth: value.google,
            kakao_oauth: value.kakao,
            git_hub_oauth: value.github,
            naver_oauth: value.naver,
        }
    }
}
#[cfg(feature = "server")]
impl UserDto {
    pub async fn update_user(self, conn: &sea_orm::DatabaseConnection) -> Result<Self, AppError> {
        use sea_orm::{ActiveModelTrait, IntoActiveModel};

        use crate::resources::entities::users;

        let model = users::Model::from(self);
        let model = model.into_active_model();

        let res = model.update(conn).await?;
        Ok(res.into())
    }

    pub async fn get_user(
        user_condition: &UserCondition,
        conn: &sea_orm::DatabaseConnection,
    ) -> Result<Vec<Self>, AppError> {
        use sea_orm::{EntityTrait, QueryFilter};

        use crate::resources::entities::users;

        let condition = user_condition.make_condition()?;

        let res = users::Entity::find().filter(condition).all(conn).await?;

        let res: Vec<UserDto> = res.into_iter().map(|m| m.into()).collect();
        Ok(res)
    }

    pub async fn create_token(
        &self,
        conn: &sea_orm::DatabaseConnection,
    ) -> Result<Tokens, AppError> {
        let (jwt, refresh) =
            crate::utils::jwt::create_token(self.id, self.username.clone(), conn).await?;

        Ok(Tokens {
            jwt,
            refresh,
            username: self.username.clone(),
            user_id: self.id,
        })
    }
}

#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
#[derive(Deserialize, Serialize, Debug)]
pub struct Tokens {
    pub jwt: String,
    pub refresh: String,
    pub username: String,
    pub user_id: i32,
}

#[cfg_attr(feature = "server", derive(utoipa::ToSchema))]
#[derive(Deserialize, Serialize)]
pub struct TokensUserId {
    pub jwt: String,
    pub refresh: String,
    pub username: String,
}

// User 인증 토큰
#[derive(Serialize, Deserialize)]
pub struct JwtClaims {
    pub exp: u64,
    pub user_id: i32,
    pub username: String,
}

// 리프레시 토큰 만료일자는 DB에 저장하기 때문에 제외
#[derive(Serialize, Deserialize)]
pub struct RefreshClaims {
    pub user_id: i32,
    pub username: String,
}

#[derive(Clone)]
pub struct CurrentUser(pub i32, pub String);
impl PartialEq for CurrentUser {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
    fn ne(&self, other: &Self) -> bool {
        self.0 != other.0
    }
}
impl PartialEq<i32> for CurrentUser {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
    fn ne(&self, other: &i32) -> bool {
        self.0 != *other
    }
}
