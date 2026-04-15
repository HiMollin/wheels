use super::config;
use sqlx::Executor;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{PgPool, Row};
use std::cmp::max;
use std::time::Duration;

pub async fn init() -> anyhow::Result<PgPool> {
    let database_config = &config::get().database();

    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        database_config.user(),
        database_config.password(),
        database_config.host(),
        database_config.port(),
        database_config.database()
    );

    let connect_options: PgConnectOptions = database_url.parse()?;
    let schema = database_config.schema().to_string();

    let cpus = num_cpus::get() as u32;
    let db = PgPoolOptions::new()
        .min_connections(max(cpus * 4, 10))
        .max_connections(max(cpus * 8, 20))
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(300))
        // 一个连接最多活 24 小时"
        .max_lifetime(Duration::from_secs(3600 * 24))
        .after_connect(move |conn, _meta| {
            let schema = schema.clone();
            Box::pin(async move {
                sqlx::query("select set_config('search_path', $1, false)")
                    .bind(&schema)
                    .execute(conn)
                    .await?;
                Ok(())
            })
        })
        .connect_with(connect_options)
        .await?;

    db.execute("select 1").await?;
    tracing::info!("Database connected successfully");

    log_database_version(&db).await?;

    Ok(db)
}

/// 打印数据库版本
async fn log_database_version(db: &PgPool) -> anyhow::Result<()> {
    let version_result = sqlx::query("select version()").fetch_one(db).await?;

    tracing::info!(
        "Database version: {}",
        version_result.try_get::<String, _>(0)?
    );

    Ok(())
}
