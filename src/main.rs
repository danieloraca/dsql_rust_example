use anyhow::Result;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_dsql::auth_token::{AuthTokenGenerator, Config};
use rand::Rng;
use rnglib::{Language, RNG};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
// use sqlx::Row;
use dotenv::dotenv;
use std::env;
use uuid::Uuid;

async fn example(cluster_endpoint: String) -> anyhow::Result<()> {
    let region = "us-east-1";
    let sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;

    let signer = AuthTokenGenerator::new(
        Config::builder()
            .hostname(&cluster_endpoint)
            .region(Region::new(region))
            .build()
            .unwrap(),
    );

    let password_token = signer
        .db_connect_admin_auth_token(&sdk_config)
        .await
        .unwrap();

    let connection_options = PgConnectOptions::new()
        .host(cluster_endpoint.as_str())
        .port(5432)
        .database("postgres")
        .username("admin")
        .password(password_token.as_str())
        .ssl_mode(sqlx::postgres::PgSslMode::VerifyFull);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect_with(connection_options.clone())
        .await?;

    //insert some rows
    for i in 0..2 {
        let id = Uuid::new_v4();
        let telephone = rand::thread_rng().gen_range(123456..987654).to_string();
        let first_name = RNG::try_from(&Language::Elven).unwrap();
        let last_name = RNG::try_from(&Language::Elven).unwrap();
        let name = format!(
            "{} {}",
            first_name.generate_name(),
            last_name.generate_name()
        );

        let result =
            sqlx::query("INSERT INTO owner (id, name, city, telephone) VALUES ($1, $2, $3, $4)")
                .bind(id)
                .bind(name)
                .bind("New York")
                .bind(telephone)
                .execute(&pool)
                .await?;

        println!("Inserted: {}", i);
        println!("Result: {:?}", result);
    }

    let rows = sqlx::query("SELECT * FROM owner WHERE name LIKE '%d%'")
        .fetch_all(&pool)
        .await?;

    let iterator_rows = rows.iter();
    for row in iterator_rows {
        println!("Row: {:?}", row);
    }

    let amount = sqlx::query("SELECT COUNT(*) FROM owner WHERE name LIKE '%d%'")
        .fetch_one(&pool)
        .await?;

    println!("Amount: {:?}", amount);

    pool.close().await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let cluster_endpoint =
        env::var("CLUSTER_ENDPOINT").expect("CLUSTER_ENDPOINT must be set in .env");

    Ok(example(cluster_endpoint.to_string()).await?)
}
