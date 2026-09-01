use std::process::exit;

use tokio_postgres::{Error, NoTls};

use crate::catalog::fetch_table_meta;
mod catalog;
#[tokio::main]
async fn main() -> Result<(), Error> {
    let e_connect = std::env::var("DATABASE_URL");
    match e_connect {
        Ok(connect_string) => {
            println!("Database Url successfully retrieved\n");
            // 1. Устанавливаем подключение
            let (client, connection) =
                tokio_postgres::connect(connect_string.as_str(), NoTls).await?;

            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("Ошибка соединения с Postgres: {}", e);
                }
            });
            let result = fetch_table_meta(&client);
            let something = result.await;

            match something {
                Ok(vec) => println!("Результат: {:#?}", vec),
                Err(err) => {
                    eprintln!("Подробная ошибка БД: {}", err);
                }
            }
        }
        Err(err) => {
            eprint!("No such a var {}\n", err);
            exit(-1);
        }
    }

    Ok(())
}
