use sea_orm::{ActiveModelTrait, ActiveValue::Set, Database, DatabaseConnection};
use topcoat::router::{Body, Router, request::Request};

use crate::{
    app, auth,
    config::{Config, Environment},
    entity::{role, user, users_roles},
};
use migration::{Migrator, MigratorTrait};

async fn setup() -> (Router, Config, DatabaseConnection) {
    let cfg = Config {
        environment: Environment::Testing,
        database_url: "sqlite::memory:".to_owned(),
        jwt_secret: "test-secret".to_owned(),
    };

    let db = Database::connect(&cfg.database_url)
        .await
        .expect("failed to connect to test database");

    Migrator::up(&db, None)
        .await
        .expect("failed to run migrations");

    let router = app::test_router(db.clone(), cfg.clone());
    (router, cfg, db)
}

async fn create_user(db: &DatabaseConnection, email: &str, admin: bool) -> user::Model {
    user::ActiveModel {
        email: Set(email.to_owned()),
        password_hash: Set("not-a-real-hash".to_owned()),
        admin: Set(admin),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("failed to create user")
}

async fn create_role(db: &DatabaseConnection, name: &str) -> role::Model {
    role::ActiveModel {
        name: Set(name.to_owned()),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("failed to create role")
}

async fn assign_role(db: &DatabaseConnection, user_id: i32, role_id: i32) {
    users_roles::ActiveModel {
        user_id: Set(user_id),
        role_id: Set(role_id),
    }
    .insert(db)
    .await
    .expect("failed to assign role");
}

fn access_token_cookie(cfg: &Config, user_id: i32) -> String {
    let tokens =
        auth::generate_tokens(user_id, &cfg.jwt_secret).expect("failed to generate tokens");
    format!("access_token={}", tokens.access_token)
}

fn request(method: &str, uri: &str, cookie: Option<&str>) -> Request {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header("cookie", cookie);
    }
    builder
        .body(Body::empty())
        .expect("failed to build request")
}

fn location(response: &topcoat::router::response::Response) -> Option<String> {
    response
        .headers()
        .get("location")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_owned())
}

#[tokio::test]
async fn protected_without_token_redirects_to_login() {
    let (router, _cfg, _db) = setup().await;
    let response = router.handle(request("GET", "/protected", None)).await;

    assert_eq!(response.status(), 303);
    assert_eq!(location(&response), Some("/login".to_owned()));
}

#[tokio::test]
async fn protected_with_token_returns_ok() {
    let (router, cfg, db) = setup().await;
    let user = create_user(&db, "user@example.com", false).await;
    let cookie = access_token_cookie(&cfg, user.id);

    let response = router
        .handle(request("GET", "/protected", Some(&cookie)))
        .await;

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn admin_without_token_redirects_to_login() {
    let (router, _cfg, _db) = setup().await;
    let response = router.handle(request("GET", "/admin", None)).await;

    assert_eq!(response.status(), 303);
    assert_eq!(location(&response), Some("/login".to_owned()));
}

#[tokio::test]
async fn admin_with_regular_user_redirects_home() {
    let (router, cfg, db) = setup().await;
    let user = create_user(&db, "user@example.com", false).await;
    let cookie = access_token_cookie(&cfg, user.id);

    let response = router.handle(request("GET", "/admin", Some(&cookie))).await;

    assert_eq!(response.status(), 303);
    assert_eq!(location(&response), Some("/".to_owned()));
}

#[tokio::test]
async fn admin_with_admin_role_returns_ok() {
    let (router, cfg, db) = setup().await;
    let user = create_user(&db, "admin-role@example.com", false).await;
    let admin_role = create_role(&db, "admin").await;
    assign_role(&db, user.id, admin_role.id).await;
    let cookie = access_token_cookie(&cfg, user.id);

    let response = router.handle(request("GET", "/admin", Some(&cookie))).await;

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn admin_with_admin_flag_returns_ok() {
    let (router, cfg, db) = setup().await;
    let user = create_user(&db, "admin-flag@example.com", true).await;
    let cookie = access_token_cookie(&cfg, user.id);

    let response = router.handle(request("GET", "/admin", Some(&cookie))).await;

    assert_eq!(response.status(), 200);
}
