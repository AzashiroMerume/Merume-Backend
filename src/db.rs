use crate::{
    models::{
        channel_model::Channel, channel_read_tracker_model::ChannelReadTracker,
        post_actioned_model::ReadPost, post_model::Post, user_channel_model::UserChannel,
        user_model::User,
    },
    responses::ErrorResponse,
};
use mongodb::{
    bson::Document,
    options::{ChangeStreamPreAndPostImages, ClientOptions, Compressor, CreateCollectionOptions},
    Client, Collection,
};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct DB {
    pub users_collection: Collection<User>,
    pub users_collection_bson: Collection<Document>,
    pub channels_collection: Collection<Channel>,
    pub channels_collection_bson: Collection<Document>,
    pub user_channels_collection: Collection<UserChannel>,
    pub user_channels_collection_bson: Collection<Document>,
    pub channel_read_trackers_collection: Collection<ChannelReadTracker>,
    pub channel_read_trackers_bson_collection: Collection<Document>,
    pub posts_collection: Collection<Post>,
    pub posts_collection_bson: Collection<Document>,
    pub read_posts_collection: Collection<ReadPost>,
    pub read_posts_collection_bson: Collection<Document>,
}

impl DB {
    pub async fn init() -> Result<Self, ErrorResponse> {
        let mongo_uri: String =
            std::env::var("MONGO_URI").expect("Failed to load `MONGO_URI` environment variable.");
        let mongo_connection_timeout: u64 = match std::env::var("MONGO_CONNECTION_TIMEOUT") {
            Ok(val) => val
                .parse()
                .expect("Failed to parse `MONGO_CONNECTION_TIMEOUT` environment variable."),
            Err(err) => panic!(
                "Failed to load `MONGO_CONNECTION_TIMEOUT` environment variable: {}",
                err
            ),
        };
        let mongo_min_pool_size: u32 = std::env::var("MONGO_MIN_POOL_SIZE")
            .expect("Failed to load `MONGO_MIN_POOL_SIZE` environment variable.")
            .parse()
            .expect("Failed to parse `MONGO_MIN_POOL_SIZE` environment variable.");
        let mongo_max_pool_size: u32 = std::env::var("MONGO_MAX_POOL_SIZE")
            .expect("Failed to load `MONGO_MAX_POOL_SIZE` environment variable.")
            .parse()
            .expect("Failed to parse `MONGO_MAX_POOL_SIZE` environment variable.");

        let db_name: String =
            std::env::var("DB_NAME").expect("Failed to load `DB_NAME` environement variable.");
        let users_collection_name: String = std::env::var("DB_USERS_TABLE")
            .expect("Failed to load `DB_USERS_TABLE` environement variable.");
        let channels_collection_name: String = std::env::var("DB_CHANNELS_TABLE")
            .expect("Failed to load `DB_CHANNELS_TABLE` environement variable.");
        let user_channels_collection_name: String = std::env::var("DB_USER_CHANNELS_TABLE")
            .expect("Failed to load `DB_USER_CHANNELS_TABLE` environement variable.");
        let channel_read_trackers_collection_name: String =
            std::env::var("DB_CHANNEL_READ_TRACKERS_TABLE")
                .expect("Failed to load `DB_CHANNEL_READ_TRACKERS_TABLE` environment variable.");
        let posts_collection_name: String = std::env::var("DB_POSTS_TABLE")
            .expect("Failed to load `DB_POSTS_TABLE` environement variable.");
        let read_posts_collection_name: String = std::env::var("DB_READ_POSTS_TABLE")
            .expect("Failed to load `DB_READ_POSTS_TABLE` environment variable.");

        let mut client_options = ClientOptions::parse(mongo_uri).await.map_err(|err| {
            eprintln!("Failed to parse MongoDB URI: {}", err);
            ErrorResponse::ServerError(None)
        })?;

        client_options.connect_timeout = Some(Duration::from_secs(mongo_connection_timeout));
        client_options.max_pool_size = Some(mongo_max_pool_size);
        client_options.min_pool_size = Some(mongo_min_pool_size);

        // the server will select the algorithm it supports from the list provided by the driver
        client_options.compressors = Some(vec![
            Compressor::Snappy,
            Compressor::Zlib {
                level: Default::default(),
            },
            Compressor::Zstd {
                level: Default::default(),
            },
        ]);

        let client = Client::with_options(client_options).map_err(|err| {
            eprintln!("Error applying options to client: {}", err);
            ErrorResponse::ServerError(None)
        })?;

        let database = client.database(db_name.as_str());

        let users_collection = database.collection::<User>(&users_collection_name);
        let users_collection_bson = database.collection::<Document>(&users_collection_name);

        let channels_collection = database.collection::<Channel>(&channels_collection_name);
        let channels_collection_bson = database.collection::<Document>(&channels_collection_name);

        let user_channels_collection =
            database.collection::<UserChannel>(&user_channels_collection_name);
        let user_channels_collection_bson =
            database.collection::<Document>(&user_channels_collection_name);

        let channel_read_trackers_collection =
            database.collection::<ChannelReadTracker>(&channel_read_trackers_collection_name);
        let channel_read_trackers_bson_collection =
            database.collection::<Document>(&channel_read_trackers_collection_name);

        let enable = ChangeStreamPreAndPostImages::builder()
            .enabled(true)
            .build();
        let opts = CreateCollectionOptions::builder()
            .change_stream_pre_and_post_images(enable)
            .build();
        let create_posts_collection = database
            .create_collection(&posts_collection_name, opts)
            .await;
        create_posts_collection.map_err(|err| {
            eprintln!(
                "Error creating posts collection with change stream options: {}",
                err
            );
            ErrorResponse::ServerError(None)
        })?;
        let posts_collection = database.collection::<Post>(&posts_collection_name);
        let posts_collection_bson = database.collection::<Document>(&posts_collection_name);

        let read_posts_collection = database.collection::<ReadPost>(&read_posts_collection_name);
        let read_posts_collection_bson =
            database.collection::<Document>(&read_posts_collection_name);

        Ok(Self {
            users_collection,
            users_collection_bson,
            channels_collection,
            channels_collection_bson,
            user_channels_collection,
            user_channels_collection_bson,
            channel_read_trackers_collection,
            channel_read_trackers_bson_collection,
            posts_collection,
            posts_collection_bson,
            read_posts_collection,
            read_posts_collection_bson,
        })
    }
}
