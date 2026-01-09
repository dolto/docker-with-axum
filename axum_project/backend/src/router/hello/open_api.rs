use axum::Router;
use utoipa::{
    Modify, OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
};
use utoipa_axum::router::OpenApiRouter;
use utoipa_redoc::{Redoc, Servable};
use utoipa_scalar::{Scalar, Servable as ScalarServable};

pub(super) const HELLO_TAG: &str = "hello";

#[derive(OpenApi)]
#[openapi(
    servers(
        (url = "/hello", description = "Hello API base path")
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = HELLO_TAG, description = "Hello handler management API")
    )
)]
pub(super) struct ApiDoc;

// Api키는 이런식으로 하지만, 서버에선 auth.rs의 SecurityAddon을 활용한다
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("hello_apikey"))),
            );
        }
    }
}

pub(super) fn set_router(base_router: OpenApiRouter) -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        // .merge(base_router)
        .nest("/hello", base_router)
        .split_for_parts();

    let router = router
        .merge(Redoc::with_url("/redoc", api.clone()))
        .merge(Scalar::with_url("/scalar", api));
    router
}
