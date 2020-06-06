use tokio_postgres::NoTls;

fn db_connection_string() -> String {
    "host=".to_owned()
        + &dotenv!("DB_HOST").to_owned()
        + " user="
        + &dotenv!("DB_USER").to_owned()
        + " password="
        + &dotenv!("DB_PASS").to_owned()
}

pub async fn get_db_client() -> tokio_postgres::Client {
    let conn = db_connection_string();
    // TODO: just for demo now, fix this later.
    info!("DB Connection to {:#?}", conn);

    let (client, connection) = tokio_postgres::connect(&conn, NoTls).await.unwrap();

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("connection error: {}", e);
        }
    });

    client
        .execute(
            "CREATE TABLE IF NOT EXISTS customers(
          id UUID PRIMARY KEY,
          name TEXT NOT NULL,
          age INT NOT NULL,
          email TEXT UNIQUE NOT NULL,
          address TEXT NOT NULL
      )",
            &[],
        )
        .await
        .expect("Could not create table");

    client
}
