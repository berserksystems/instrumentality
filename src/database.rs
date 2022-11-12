//! Database functions and implementations for Instrumentality.

use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::Response;
use mongodb::bson::Document;
use mongodb::options::IndexOptions;
use mongodb::results::CreateIndexResult;
use mongodb::ClientSession;
use mongodb::{bson::doc, Client, Collection, Database, IndexModel};

use crate::config::IConfig;
use crate::data::Data;
use crate::subject::Subject;
use crate::user::User;

#[derive(Clone)]
pub struct DBPool {
    client: Client,
    database: String,
}

impl DBPool {
    pub async fn handle_with_started_transaction(&self) -> DBHandle {
        let mut session = self.client.start_session(None).await.unwrap();
        session.start_transaction(None).await.unwrap();
        DBHandle {
            db: self.client.database(&self.database),
            session,
        }
    }

    pub async fn handle(&self) -> DBHandle {
        let session = self.client.start_session(None).await.unwrap();
        DBHandle {
            db: self.client.database(&self.database),
            session,
        }
    }
}

pub struct DBHandle {
    db: Database,
    pub session: ClientSession,
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
    let mongo_client =
        Client::with_options(config.mongodb.client_opts()).unwrap();
    let database = mongo_client.database(&config.mongodb.database);

    // It is only at this point that MongoDB actually makes a connection.
    database
        .run_command(doc! {"ping" : 1_u32}, None)
        .await
        .expect("Couldn't connect to MongoDB");

    tracing::info!("Connected to MongoDB.");

    let is_fresh_install = !mongo_client
        .list_database_names(None, None)
        .await
        .unwrap()
        .contains(&config.mongodb.database);

    if is_fresh_install {
        let root_user = create_root_account(&database).await.unwrap();
        tracing::info!("Created root account.");
        tracing::info!("\n{:#?}", root_user);
        create_indexes(&database).await;
        tracing::info!("Created MongoDB indices.")
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
    let user = User {
        admin: true,
        ..User::new("root")
    };
    users_coll.insert_one(&user, None).await.unwrap();
    Ok(user)
}

async fn create_indexes(database: &Database) {
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
impl<S> FromRequestParts<S> for DBHandle
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let db_pool = parts.extensions.get::<DBPool>().unwrap();

        let db = db_pool.handle_with_started_transaction().await;

        Ok(db)
    }
}
