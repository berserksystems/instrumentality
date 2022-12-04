//! Error responses.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub response: String,
    pub text: String,
}

impl ErrorResponse {
    pub fn from_text(text: &str) -> Self {
        Self {
            response: "ERROR".to_string(),
            text: text.to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Default)]
pub struct OkResponse {
    pub response: String,
}

impl OkResponse {
    pub fn new() -> Self {
        Self {
            response: "OK".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct InviteResponse {
    pub response: String,
    pub code: String,
}

impl InviteResponse {
    pub fn new(code: String) -> Self {
        Self {
            response: "OK".to_string(),
            code,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct QueueResponse {
    pub response: String,
    pub queue_id: String,
    pub platform: String,
    pub platform_id: String,
    pub platform_username_hint: String,
}

impl QueueResponse {
    pub fn new(
        queue_id: String,
        platform: String,
        platform_id: String,
        platform_username_hint: String,
    ) -> Self {
        Self {
            response: "OK".to_string(),
            queue_id,
            platform,
            platform_id,
            platform_username_hint,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct UserResponse {
    pub response: String,
    pub user: crate::user::User,
}

impl UserResponse {
    pub fn from_user(user: crate::user::User) -> Self {
        Self {
            response: "OK".to_string(),
            user,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ViewResponse {
    pub response: String,
    pub view_data: crate::routes::view::ViewData,
}

impl ViewResponse {
    pub fn from_view_data(view_data: crate::routes::view::ViewData) -> Self {
        Self {
            response: "OK".to_string(),
            view_data,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct TypesResponse {
    pub response: String,
    pub content_types: std::collections::HashMap<String, Vec<String>>,
    pub presence_types: std::collections::HashMap<String, Vec<String>>,
}

impl TypesResponse {
    pub fn from_types(
        content_types: std::collections::HashMap<String, Vec<String>>,
        presence_types: std::collections::HashMap<String, Vec<String>>,
    ) -> Self {
        Self {
            response: "OK".to_string(),
            content_types,
            presence_types,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ResetResponse {
    pub response: String,
    pub key: String,
}

impl ResetResponse {
    pub fn from_key(key: String) -> Self {
        Self {
            response: "OK".to_string(),
            key,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub response: String,
    pub user: crate::user::User,
    pub subjects: Vec<crate::subject::Subject>,
    pub groups: Vec<crate::group::Group>,
}

impl LoginResponse {
    pub fn from_user_data(
        user: crate::user::User,
        subjects: Vec<crate::subject::Subject>,
        groups: Vec<crate::group::Group>,
    ) -> Self {
        Self {
            response: "OK".to_string(),
            user,
            subjects,
            groups,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct CreateResponse {
    pub response: String,
    pub uuid: String,
}

impl CreateResponse {
    pub fn from_uuid(uuid: &str) -> Self {
        Self {
            response: "OK".to_string(),
            uuid: uuid.to_string(),
        }
    }
}

macro_rules! ok {
    () => {
        ok!(OK)
    };
    ($code:ident) => {
        Ok(response!($code, OkResponse::new()))
    };
    ($code:ident, $response:expr) => {
        Ok(response!($code, $response))
    };
}

macro_rules! error {
    ($code:ident, $text:expr) => {
        Err(response!($code, ErrorResponse::from_text($text)))
    };
}

macro_rules! response {
    ($code:ident, $response: expr) => {
        (StatusCode::$code, Json($response))
    };
}
