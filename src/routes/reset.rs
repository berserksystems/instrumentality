//! Route for resetting an API key for Instrumentality.
//!
//! The /reset route is implemented here.
//!
//! See endpoint documentation at
//! <https://docs.berserksystems.com/endpoints/reset/>.

use axum::{http::StatusCode, response::IntoResponse, Json};
use mongodb::bson::doc;

use crate::database::DBHandle;
use crate::routes::response::{ErrorResponse, ResetResponse};
use crate::user::User;

pub async fn reset(user: User, mut db: DBHandle) -> impl IntoResponse {
    let new_key = User::new_key();
    let u_coll = db.collection::<User>("users");
    u_coll
        .find_one_and_update_with_session(
            doc! {"key": user.key},
            doc! { "$set": {"key": &new_key}},
            None,
            &mut db.session,
        )
        .await
        .unwrap();
    let result = db.session.commit_transaction().await;
    match result {
        Ok(_) => ok!(OK, ResetResponse::from_key(new_key)),
        _ => error!(
            INTERNAL_SERVER_ERROR,
            "Could not reset key. Your key remains the same. Please try again."
        ),
    }
}
