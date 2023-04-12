use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use bson::{doc, oid::ObjectId};
use mongodb::{Client, Collection};

use crate::{models::user_model::User, responses::main_response::MainResponse};

pub async fn get_preferences(
    State(client): State<Client>,
    Extension(user_id): Extension<ObjectId>,
) -> impl IntoResponse {
    let collection: Collection<User> = client.database("Merume").collection("users");

    let user = match collection.find_one(doc! {"_id": user_id}, None).await {
        Ok(Some(user)) => user,
        _ => {
            eprintln!("Error finding user");
            let main_response = MainResponse {
                success: false,
                data: None,
                error_message: Some(
                    "There was an error on the server side, try again later.".to_string(),
                ),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(main_response));
        }
    };

    let main_response = MainResponse {
        success: true,
        data: Some(user.preferences.unwrap()),
        error_message: None,
    };

    (StatusCode::OK, Json(main_response))
}
