use tokio_postgres::{Error, NoTls};

use crate::catalog::fetch_table_meta;
mod catalog;
#[tokio::main]
async fn main() -> Result<(), Error> {
    let connect_str = "host=localhost user=kiryloshakirov  dbname=postgres_air";

    // 1. Устанавливаем подключение
    let (client, connection) = tokio_postgres::connect(connect_str, NoTls).await?;

    // 2. Spawn таски для обслуживания цикла ввода-вывода (IO Connection Loop)

    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Ошибка соединения с Postgres: {}", e);
        }
    });
    let  result  =  fetch_table_meta(&client);
    let something = result.await;

    match something {
    Ok(vec) => println!("Результат: {}", vec.len()),
    Err(err) =>  println!("Произошла ошибка: {}", err),
    }

    if let Err(e) = fetch_table_meta(&client).await {
    eprintln!("Подробная ошибка БД: {:?}", e);
}

    
    // println!("Hello, world!");
    // let rows = client.query("SELECT 1 AS ping", &[]).await?;
    // let val: i32 = rows[0].get(0);
    // println!("Ответ от сервера (ping): {}", val);
    Ok(())

	
}
