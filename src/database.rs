//! Database functions and implementations for Instrumentality.

use crate::config::IConfig;
use crate::data::Data;
use crate::subject::Subject;
use crate::user::User;

use axum::async_trait;
use axum::extract::{FromRequest, RequestParts};
use axum::response::Response;
use mongodb::bson::Document;
use mongodb::options::{ClientOptions, IndexOptions};
use mongodb::results::CreateIndexResult;
use mongodb::{bson::doc, Client, Collection, Database, IndexModel};
use std::time::Duration;

#[derive(Clone)]
pub struct DBPool {
    client: Client,
    database: String,
}

impl DBPool {
    pub fn handle(&self) -> DBHandle {
        DBHandle {
            db: self.client.database(&self.database),
        }
    }
}

pub struct DBHandle {
    db: Database,
}

impl DBHandle {
    pub fn collection<T>(&self, name: &str) -> Collection<T> {
        self.db.collection::<T>(name)
    }

    pub async fn drop(&self) -> Result<(), mongodb::error::Error> {
        self.db.drop(None).await
    }
}

pub async fn open(
    config: &IConfig,
) -> Result<DBPool, Box<dyn std::error::Error>> {
    let user = &config.mongodb.user;
    let password = &config.mongodb.password;
    let hosts = &config.mongodb.hosts;
    let port = &config.mongodb.port;
    let database = &config.mongodb.database;
    let mut mongo_options = ClientOptions::parse(format!(
        "mongodb://{user}:{password}@{hosts}:{port}"
    ))
    .await?;
    mongo_options.connect_timeout = Some(Duration::new(1, 0));
    mongo_options.heartbeat_freq = Some(Duration::new(1, 0));
    mongo_options.server_selection_timeout = Some(Duration::new(1, 0));
    let mongo_client = Client::with_options(mongo_options).unwrap();
    let database = mongo_client.database(database);

    // It is only at this point that MongoDB actually makes a connection.
    database
        .run_command(doc! {"ping" : 1_u32}, None)
        .await
        .expect("Couldn't connect to MongoDB");

    let user_count: u64 = database
        .collection::<User>("users")
        .count_documents(None, None)
        .await
        .unwrap();
    if user_count == 0 {
        tracing::info!("Creating root account...");
        let root_user = create_root_account(&database).await.unwrap();
        tracing::info!("\n{:#?}", root_user);
        tracing::info!("Creating indexes...");
        create_indexes(&database).await;
    }

    Ok(DBPool {
        client: mongo_client,
        database: config.mongodb.database.to_string(),
    })
}

async fn create_root_account(
    database: &Database,
) -> Result<User, Box<dyn std::error::Error>> {
    let users_coll: Collection<User> = database.collection("users");
    let user = User::new("root");
    users_coll.insert_one(&user, None).await.unwrap();
    Ok(user)
}

async fn create_indexes(database: &Database) {
    unique_content_index(database).await.unwrap();
    unique_subject_name_index(database).await.unwrap();
    create_index("Users Key Index", "users", doc! {"key" : 1_u32}, database)
        .await
        .unwrap();
    create_index(
        "Users Key & Banned Index",
        "users",
        doc! {"key" : 1_u32, "banned" : 1_u32},
        database,
    )
    .await
    .unwrap();
    create_index(
        "Queue Platform & Platform ID",
        "queue",
        doc! {"platform" : 1_u32, "platform_id" : 1_u32},
        database,
    )
    .await
    .unwrap();
    create_index(
        "Queue ID Index",
        "queue",
        doc! {"queue_id" : 1_u32},
        database,
    )
    .await
    .unwrap();
    create_index(
        "Subjects UUID Index",
        "subjects",
        doc! {"uuid" : 1_u32},
        database,
    )
    .await
    .unwrap();
    create_index(
        "Groups UUID Index",
        "groups",
        doc! {"uuid" : 1_u32},
        database,
    )
    .await
    .unwrap();
    create_index(
        "Referrals Code Used Index",
        "referrals",
        doc! {"code" : 1_u32, "used" : 1_u32},
        database,
    )
    .await
    .unwrap();
    create_index(
        "Data Platform ID & Platform Index",
        "data",
        doc! {"id" : 1_u32, "platform" : 1_u32},
        database,
    )
    .await
    .unwrap();
    create_index(
        "Queue Platform & Platform ID",
        "queue",
        doc! {"platform" : 1_u32, "platform_id" : 1_u32},
        database,
    )
    .await
    .unwrap();
}

async fn unique_subject_name_index(
    database: &Database,
) -> Result<CreateIndexResult, mongodb::error::Error> {
    let idx_options = IndexOptions::builder()
        .name(String::from("Unique Subject Name"))
        .unique(true)
        .build();

    let idx_model = IndexModel::builder()
        .keys(doc! {"created_by" : 1_u32, "name": 1_u32})
        .options(idx_options)
        .build();

    database
        .collection::<Subject>("subjects")
        .create_index(idx_model, None)
        .await
}

async fn unique_content_index(
    database: &Database,
) -> Result<CreateIndexResult, mongodb::error::Error> {
    let idx_options = IndexOptions::builder()
        .name(String::from("Unique Content ID"))
        .unique(true)
        .sparse(true)
        .build();

    let idx_model = IndexModel::builder()
        .keys(doc! {"content_id" : 1_u32,
        "platform": 1_u32,
        "content_type" : 1_u32})
        .options(idx_options)
        .build();

    database
        .collection::<Data>("data")
        .create_index(idx_model, None)
        .await
}

async fn create_index(
    index_name: &str,
    collection_name: &str,
    keys: Document,
    database: &Database,
) -> Result<CreateIndexResult, mongodb::error::Error> {
    let idx_options =
        IndexOptions::builder().name(index_name.to_string()).build();

    let idx_model = IndexModel::builder()
        .keys(keys)
        .options(idx_options)
        .build();

    database
        .collection::<Data>(collection_name)
        .create_index(idx_model, None)
        .await
}

pub async fn drop_database(database: &DBHandle) {
    database.drop().await.unwrap();
}

#[async_trait]
impl<B: Send> FromRequest<B> for DBHandle {
    type Rejection = Response;

    async fn from_request(
        request: &mut RequestParts<B>,
    ) -> Result<Self, Self::Rejection> {
        let db_pool = request.extensions().get::<DBPool>().unwrap();

        let db = db_pool.handle();

        Ok(db)
    }
}
