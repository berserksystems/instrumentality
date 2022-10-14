//! Error responses.
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Error {
    pub response: String,
    pub text: String,
}

impl Error {
    pub fn new(text: &str) -> Self {
        Self {
            response: "ERROR".to_string(),
            text: text.to_string(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Ok {
    pub response: String,
}

impl Ok {
    pub fn new() -> Self {
        Self {
            response: "OK".to_string(),
        }
    }
}

impl Default for Ok {
    fn default() -> Self {
        Self::new()
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
pub struct RegisterResponse {
    pub response: String,
    pub user: crate::user::User,
}

impl RegisterResponse {
    pub fn new(user: crate::user::User) -> Self {
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
    pub fn new(view_data: crate::routes::view::ViewData) -> Self {
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
    pub fn new(
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
    pub new_key: String,
}

impl ResetResponse {
    pub fn new(new_key: String) -> Self {
        Self {
            response: "OK".to_string(),
            new_key,
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
    pub fn new(
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
    pub fn new(uuid: &str) -> Self {
        Self {
            response: "OK".to_string(),
            uuid: uuid.to_string(),
        }
    }
}
