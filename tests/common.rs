//! Always use `prepare_environment` unless you are certain of what you are
//! doing. Using it twice will not create another user. Use the given user
//! to create an invite via /invite and register that user.
//!
//! `setup_client` starts instrumentality, which will connect to mongodb on
//! startup and check if the user collection is empty (shorthand for is this
//! a fresh database). If so, a root account is created as are some indexes.
//!
//! It is VITAL that you do not call `inject_test_account` before setting up
//! the client. If you do this, the indexes enforcing uniqueness on collections
//! will not be created and your test environment will yield subtly different
//! outcomes making debugging difficult.

use axum::body::Body;
use axum::http::Method;
use axum::http::Request;
use axum::http::StatusCode;
use axum::Router;
use axum_server::Handle;
use chrono::Utc;
use instrumentality::config;
use instrumentality::config::IConfig;
use instrumentality::data::Data;
use instrumentality::database;
use instrumentality::response::LoginResponse;
use instrumentality::server;
use instrumentality::user::User;
use tower::Service;
use uuid::Uuid;

pub const TEST_ENVIRONMENT_CONFIG: &str = "InstrumentalityTest.toml";

pub struct Environment {
    pub app: Router,
    pub user: User,
    pub config: IConfig,
    pub handle: Handle,
}

impl Environment {
    pub async fn default() -> Self {
        Self::new(TEST_ENVIRONMENT_CONFIG).await
    }

    // TODO: add an attribute macro that calls new and cleanup for a test.
    pub async fn new(config_path: &str) -> Self {
        let mut config = config::open(config_path).unwrap();
        let test_db_id = Uuid::new_v4().to_string();
        config.mongodb.database = test_db_id.clone();
        let (app, _, _, handle) = server::build_server(&config).await;

        let user = User::new("test");
        Self::inject_account(&config, &user).await;

        Self {
            app,
            user,
            config,
            handle,
        }
    }

    pub async fn cleanup(&self) {
        let database = database::open(&self.config).await.unwrap();
        database::drop_database(&database.handle()).await;
    }

    // This is only used in tests, so it flags as dead code.
    #[allow(dead_code)]
    pub async fn login(&mut self) -> LoginResponse {
        let res = self
            .app
            .call(
                Request::builder()
                    .method(Method::GET)
                    .header("X-API-KEY", &self.user.key)
                    .uri("/login")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let lr: LoginResponse = serde_json::from_slice(&body).unwrap();

        lr
    }

    pub async fn inject_account(config: &IConfig, user: &User) {
        let database = database::open(&config).await.unwrap();

        let _user_coll = database
            .handle()
            .collection::<User>("users")
            .insert_one(user, None)
            .await
            .unwrap();
    }
}

#[allow(dead_code)]
pub fn create_mock_content(id: &str, platform: &str) -> Data {
    Data::Content {
        id: id.to_string(),
        platform: platform.to_string(),
        content_type: "story".to_string(),
        retrieved_at: Utc::now(),
        content_id: Uuid::new_v4().to_string(),
        deleted: Some(false),
        retrieved_from: None,
        created_at: None,
        body: None,
        media: None,
        references: None,
        added_by: None,
        added_at: None,
    }
}

#[allow(dead_code)]
pub fn create_mock_presence(id: &str, platform: &str) -> Data {
    Data::Presence {
        id: id.to_string(),
        platform: platform.to_string(),
        presence_type: "live".to_string(),
        retrieved_at: Utc::now(),
        added_by: None,
        added_at: None,
    }
}
