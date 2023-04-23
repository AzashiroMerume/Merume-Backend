use crate::{handlers, middlewares};
use handlers::{
    auth_handlers, channels_handlers, channels_handlers::user_channels_handlers,
    content_recomendation_handlers, posts_handlers, preferred_content_handlers,
};
use middlewares::{auth_middleware, verify_channel_owner_middleware};

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use mongodb::Client;
use tower_http::limit::RequestBodyLimitLayer;

pub fn auth_routes(client: Client) -> Router<Client> {
    Router::new()
        .route("/", get(auth_handlers::verify_auth_handler::verify_auth))
        .route_layer(middleware::from_fn_with_state(
            client,
            |state, req, next| auth_middleware::auth(state, req, next, Some(false)),
        ))
        .route("/register", post(auth_handlers::register_handler::register))
        .route("/login", post(auth_handlers::login_handler::login))
        .layer(RequestBodyLimitLayer::new(1024))
}

pub fn user_channels_routes(client: Client) -> Router<Client> {
    Router::new()
        .route(
            "/subscriptions",
            get(user_channels_handlers::subscribed_channels_handler::subscribed_channels),
        )
        .route(
            "/created",
            get(user_channels_handlers::created_channels_handler::created_channels),
        )
        .route(
            "/new",
            post(user_channels_handlers::new_channel_handler::new_channel),
        )
        .layer(middleware::from_fn_with_state(
            client,
            |state, req, next| auth_middleware::auth(state, req, next, Some(false)),
        ))
        .layer(RequestBodyLimitLayer::new(1024))
}

pub fn channels_routes(client: Client) -> Router<Client> {
    Router::new()
        .route(
            "/:channel_id",
            get(channels_handlers::get_channel_handler::get_channel_by_id),
        )
        .route(
            "/:channel_id/subscribe",
            get(channels_handlers::subscribe_to_channel_handler::subscribe_to_channel),
        )
        .layer(middleware::from_fn_with_state(
            client,
            |state, req, next| auth_middleware::auth(state, req, next, Some(false)),
        ))
}

pub fn post_routes(client: Client) -> Router<Client> {
    Router::new()
        .route(
            "/:channel_id",
            post(posts_handlers::create_post_handler::create_post),
        )
        .layer(middleware::from_fn_with_state(
            client.clone(),
            verify_channel_owner_middleware::verify_channel_owner,
        ))
        .layer(middleware::from_fn_with_state(
            client,
            |state, req, next| auth_middleware::auth(state, req, next, Some(false)),
        ))
        .layer(RequestBodyLimitLayer::new(1024))
}

pub fn channel_system(client: Client) -> Router<Client> {
    Router::new()
        .merge(channels_routes(client.clone()))
        .merge(post_routes(client))
}

pub fn recomendations_routes(client: Client) -> Router<Client> {
    Router::new()
        .route(
            "/recomendations",
            get(content_recomendation_handlers::recomendations_handler::recomendations),
        )
        .layer(middleware::from_fn_with_state(
            client,
            |state, req, next| auth_middleware::auth(state, req, next, Some(true)),
        ))
}

pub fn preferred_content_routes(client: Client) -> Router<Client> {
    Router::new()
        .route(
            "/",
            get(preferred_content_handlers::get_preferences_handler::get_preferences),
        )
        .route_layer(middleware::from_fn_with_state(
            client.clone(),
            |state, req, next| auth_middleware::auth(state, req, next, Some(true)),
        ))
        .route(
            "/",
            post(preferred_content_handlers::post_preferences_handler::post_preferences),
        )
        .route_layer(middleware::from_fn_with_state(
            client,
            |state, req, next| auth_middleware::auth(state, req, next, Some(false)),
        ))
        .layer(RequestBodyLimitLayer::new(1024))
}

// let auth_routes = Router::new()
//     .route("/", get(auth_handlers::verify_auth_handler::verify_auth))
//     .route_layer(middleware::from_fn_with_state(
//         client.clone(),
//         |state, req, next| auth_middleware::auth(state, req, next, Some(false)),
//     ))
//     .route("/register", post(auth_handlers::register_handler::register))
//     .route("/login", post(auth_handlers::login_handler::login));

// let user_channels_routes = Router::new()
//     .route(
//         "/subscriptions",
//         get(user_channels_handlers::subscribed_channels_handler::subscribed_channels),
//     )
//     .route(
//         "/created",
//         get(user_channels_handlers::created_channels_handler::created_channels),
//     )
//     .route(
//         "/new",
//         post(user_channels_handlers::new_channel_handler::new_channel),
//     )
//     .layer(middleware::from_fn_with_state(
//         client.clone(),
//         |state, req, next| auth_middleware::auth(state, req, next, Some(false)),
//     ));

// let channels_routes = Router::new()
//     .route(
//         "/:channel_id",
//         get(channels_handlers::get_channel_handler::get_channel_by_id),
//     )
//     .route(
//         "/:channel_id/subscribe",
//         get(channels_handlers::subscribe_to_channel_handler::subscribe_to_channel),
//     )
//     .layer(middleware::from_fn_with_state(
//         client.clone(),
//         |state, req, next| auth_middleware::auth(state, req, next, Some(false)),
//     ));

// let post_routes = Router::new()
//     .route(
//         "/:channel_id",
//         post(posts_handlers::create_post_handler::create_post),
//     )
//     .layer(middleware::from_fn_with_state(
//         client.clone(),
//         verify_channel_owner_middleware::verify_channel_owner,
//     ))
//     .layer(middleware::from_fn_with_state(
//         client.clone(),
//         |state, req, next| auth_middleware::auth(state, req, next, Some(false)),
//     ));

// let channel_system = Router::new().merge(channels_routes).merge(post_routes);

// let preferred_content_routes = Router::new()
//     .route(
//         "/",
//         get(preferred_content_handlers::get_preferences_handler::get_preferences),
//     )
//     .route_layer(middleware::from_fn_with_state(
//         client.clone(),
//         |state, req, next| auth_middleware::auth(state, req, next, Some(true)),
//     ))
//     .route(
//         "/",
//         post(preferred_content_handlers::post_preferences_handler::post_preferences),
//     )
//     .route_layer(middleware::from_fn_with_state(
//         client.clone(),
//         |state, req, next| auth_middleware::auth(state, req, next, Some(false)),
//     ));
