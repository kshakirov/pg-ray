use std::process::exit;

use tokio_postgres::{Error, NoTls};

use crate::catalog::{fetch_table_meta, ColumnStats};
mod catalog;

fn compute_prob(freqs: Vec<Vec<f32>>) {
    print!("Computing the probability...\n");
}
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
                Ok(vec) => {
                    println!("Результат: {:#?}", vec.len());
                    for table in vec {
                        println!("Processing Table [{}]", table.table_name);
                        let cols = table.columns;
                        for cc in cols {
                            if cc.mcf.is_some() {
                                println!("\t Column[{}]", cc.column_name);
                                let v = cc.mcf.iter().fold(0.0, |acc, f| {
                                    acc + f.iter().fold(0.0, |aacc, ff| aacc + ff)
                                });
                                println!(
                                    "The frequency is {:#?} the other probability is {} ",
                                    v,
                                    1.0 - v
                                );
                            }
                        }
                    }
                    //compute_prob();
                }
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
