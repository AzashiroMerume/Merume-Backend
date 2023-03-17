use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use bson::{oid::ObjectId, Bson};
use futures::StreamExt;
use mongodb::{bson::doc, options::FindOptions, Client, Collection};

use crate::models::channels_model::Channels;
use crate::responses::main_response::MainResponse;

pub async fn get_channels(State(client): State<Client>) -> impl IntoResponse {
    let coll = client.database("Merume").collection::<Channels>("channels");
    let options = FindOptions::default();

    let mut cursor = coll
        .find(None, options)
        .await
        .expect("could not load channels data.");

    let mut channels: Vec<Channels> = Vec::new();

    while let Some(doc) = cursor.next().await {
        channels.push(doc.expect("could not load channels info."));
    }

    let response = MainResponse {
        success: true,
        data: Some(channels),
        error_message: None,
    };

    (StatusCode::NOT_FOUND, Json(response))
}

pub async fn get_channels_by_id(
    State(client): State<Client>,
    Path(channel_id): Path<String>,
) -> impl IntoResponse {
    let id = ObjectId::parse_str(channel_id);

    if let Err(err) = id {
        return (
            StatusCode::BAD_REQUEST,
            Json(MainResponse {
                success: false,
                error_message: Some(format!("Invalid value provided for id, reason: {:#?}", err)),
                data: None,
            }),
        );
    }

    let channels_coll: Collection<Channels> =
        client.database("Merume").collection::<Channels>("channels");

    // let mut options = FindOneOptions::default();

    let filter = doc! {
        "_id": Some(id.unwrap())
    };
    let channel = channels_coll.find_one(filter.clone(), None).await;
    match channel {
        Ok(value) => {
            match value {
                Some(channel) => {
                    return (
                        StatusCode::FOUND,
                        Json(MainResponse {
                            success: true,
                            data: Some(vec![channel]),
                            error_message: None,
                        }),
                    );
                }
                None => {
                    let mut message: String = "".to_owned();
                    for (k, v) in filter {
                        let message_part = match v {
                            Bson::String(val) => format!("{}=={}, ", k, val),
                            _ => format!("{}=={}, ", k, v),
                        };
                        message.push_str(&message_part);
                    }
                    return (
                        StatusCode::NOT_FOUND,
                        Json(MainResponse {
                            success: false,
                            error_message: Some(format!(
                                "No channel exists for given filter: {}",
                                message
                            )),
                            data: None,
                        }),
                    );
                }
            };
        }
        Err(err) => (
            StatusCode::NOT_FOUND,
            Json(MainResponse {
                success: false,
                error_message: Some(format!("Couldn't find any channel due to {:#?}", err)),
                data: None,
            }),
        ),
    }
}
