use futures::TryStreamExt;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use eyre::Result;
use sqlx::{
    migrate::{MigrateDatabase, Migrator},
    Row, Sqlite, SqlitePool,
};

use tracing::info;
use yt_sub_core::UserSettings;

//TODO lazy default ENV
static LITE_DB_URL: &str = "sqlite://ytsub.db";
static MIGRATOR: Migrator = sqlx::migrate!();

#[derive(Debug, sqlx::FromRow, PartialEq)]
pub struct UserRow {
    pub id: String,
    pub settings_json: String,
    pub last_run_at: Option<DateTime<Utc>>,
}

pub async fn sqlite_conn(db_url: Option<&str>) -> Result<Arc<SqlitePool>> {
    let db_url = db_url.unwrap_or(LITE_DB_URL);

    match SqlitePool::connect(db_url).await {
        Ok(conn) => Ok(Arc::new(conn)),
        Err(error) => eyre::bail!("Error connecting to db: {}", error),
    }
}

pub async fn init_lite_db(db_url: Option<&str>) -> Result<()> {
    let db_url = db_url.unwrap_or(LITE_DB_URL);

    if Sqlite::database_exists(db_url).await.unwrap_or(false) {
        info!("Database {} already exists", db_url);
        return Ok(());
    }

    info!("Creating database {}", db_url);
    match Sqlite::create_database(db_url).await {
        Ok(_) => {
            info!("Create {} db success", &db_url);
            // Run migrations
            let db = SqlitePool::connect(db_url).await?;
            match MIGRATOR.run(&db).await {
                Ok(_) => info!("Migrations run successfully"),
                Err(error) => panic!("Failed to run migrations: {}", error),
            }
        }
        Err(error) => panic!("error: {}", error),
    }

    Ok(())
}

impl UserRow {
    pub async fn ids(conn: &SqlitePool) -> Result<Vec<String>> {
        let mut ids = vec![];

        let mut rows = sqlx::query("SELECT id FROM users").fetch(conn);

        while let Ok(row) = rows.try_next().await {
            let Some(row) = row else {
                break;
            };

            let id: String = row.get("id");
            ids.push(id);
        }

        Ok(ids)
    }

    pub async fn new(settings: UserSettings, last_run_at: Option<DateTime<Utc>>) -> Result<Self> {
        let api_key = settings.api_key.clone().expect("Missing API key");

        Ok(Self {
            id: api_key,
            settings_json: settings.to_string(),
            last_run_at,
        })
    }

    pub async fn exists(id: &str, conn: &SqlitePool) -> Result<bool> {
        let exists = sqlx::query("SELECT EXISTS(SELECT 1 FROM users WHERE id = ?)")
            .bind(id)
            .fetch_one(conn)
            .await?
            .get::<bool, _>(0);

        Ok(exists)
    }

    pub async fn save(&self, conn: &SqlitePool) -> Result<()> {
        if Self::exists(self.id.as_str(), conn).await? {
            sqlx::query("UPDATE users SET settings_json = ?, last_run_at = ? WHERE id = ?")
                .bind(self.settings_json.to_string())
                .bind(self.last_run_at)
                .bind(self.id.clone())
                .execute(conn)
                .await?;
        } else {
            sqlx::query("INSERT INTO users (id, settings_json, last_run_at) VALUES (?, ?, ?)")
                .bind(self.id.clone())
                .bind(self.settings_json.to_string())
                .bind(self.last_run_at)
                .execute(conn)
                .await?;
        }

        Ok(())
    }

    pub async fn get(id: &str, conn: &SqlitePool) -> Result<Option<Self>> {
        let row = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(conn)
            .await?;

        Ok(row)
    }

    pub async fn delete(id: &str, conn: &SqlitePool) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id)
            .execute(conn)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use uuid::Uuid;

    use crate::controllers::account::tests::build_settings;

    use super::*;
    use std::fs;

    pub async fn setup_test_db() -> (Arc<SqlitePool>, SqliteCleaner) {
        let uuid = Uuid::new_v4();
        let db_path = format!("/tmp/{}-ytsub.db", uuid);
        let db_url = format!("sqlite://{}", db_path);

        if fs::remove_file(&db_url).is_ok() {
            println!("DB {} removed", &db_url);
        }

        init_lite_db(Some(&db_url))
            .await
            .expect("Failed to init db");

        let cleaner = SqliteCleaner {
            db_uuid: uuid.to_string(),
        };

        (
            sqlite_conn(Some(&db_url))
                .await
                .expect("Failed to connect to db"),
            cleaner,
        )
    }

    pub struct SqliteCleaner {
        pub db_uuid: String,
    }

    impl Drop for SqliteCleaner {
        fn drop(&mut self) {
            let pattern = format!("/tmp/*{}*", self.db_uuid);

            for entry in glob::glob(&pattern).expect("Failed to read glob pattern") {
                match entry {
                    Ok(path) => {
                        if let Err(e) = fs::remove_file(&path) {
                            eprintln!("Failed to remove file {:?}: {}", path, e);
                        }
                    }
                    Err(e) => eprintln!("Error reading glob entry: {}", e),
                }
            }
        }
    }

    #[tokio::test]
    async fn create_remove_user() -> Result<()> {
        let (conn, _cl) = setup_test_db().await;
        let uuid = Uuid::new_v4().to_string();
        let exists = UserRow::exists(&uuid, &conn).await?;
        assert!(!exists);

        let settings = build_settings(true, Some("https://slack.com/webhook".to_string()));
        let user = UserRow::new(settings, None).await?;

        user.save(&conn).await?;
        let exists = UserRow::exists(&user.id, &conn).await?;

        assert!(exists);

        let user = UserRow::get(&user.id, &conn)
            .await?
            .expect("User not found");

        UserRow::delete(&user.id, &conn).await?;

        let exists = UserRow::exists(&user.id, &conn).await?;
        assert!(!exists);
        Ok(())
    }
}
