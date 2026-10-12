use std::process::exit;

use tokio_postgres::{Error, NoTls};

use crate::catalog::{TableMeta, fetch_table_meta, get_mcf_by_mcv};
mod catalog;

fn compute_prob(tables: Vec<TableMeta>) {
    print!("Computing the probability...\n");
    for table in tables {
        println!("Processing Table [{}]", table.table_name);
        let cols = table.columns;
        for cc in cols {
            if cc.mcf.is_some() {
                println!("\t Column[{}]", cc.column_name);
                let null_frac = cc.null_frac;
                let v = cc
                    .mcf
                    .iter()
                    .fold(0.0, |acc, f| acc + f.iter().fold(0.0, |aacc, ff| aacc + ff));
                println!(
                    "The frequency is {:#?}, the null_frac is {}, the other probability is {} ",
                    v,
                    null_frac,
                    1.0 - v - null_frac
                );
            }
        }
    }
}

enum Plan {
    SeqScan,
}
fn cost(p: Plan) -> f64 {
    42.0
}
fn seq_scan_disk_cost(relpages: f64, seq_page_cost: f64) -> f64 {
    relpages * seq_page_cost
}

fn cpu_costs(reltuples: f64, cpu_tuples_cost: f64) -> f64 {
    reltuples * cpu_tuples_cost
}
#[tokio::main]
async fn main() {
    let relpages: f64 = 8608.0;
    let seq_page_cost: f64 = 1.0;
    let costs = cost(Plan::SeqScan);
    let reltuples: f64 = 683178.0;
    let cpu_tuples_cost: f64 = 0.01;
    let ssdc = seq_scan_disk_cost(relpages, seq_page_cost);
    let cc = cpu_costs(reltuples, cpu_tuples_cost);
    let ph_costs = cc + ssdc;

    println!(
        "Planner costs {} and seq cost {}  cpu costs {}  all phys costs are {}\n",
        costs, ssdc, cc, ph_costs
    );

    // let e_connect = std::env::var("DATABASE_URL");
    // match e_connect {
    //     Ok(connect_string) => {
    //         println!("Database Url successfully retrieved\n");
    //         // 1. Устанавливаем подключение
    //         let (client, connection) =
    //             tokio_postgres::connect(connect_string.as_str(), NoTls).await?;

    //         tokio::spawn(async move {
    //             if let Err(e) = connection.await {
    //                 eprintln!("Ошибка соединения с Postgres: {}", e);
    //             }
    //         });
    //         let result = fetch_table_meta(&client);
    //         let something = result.await;
    //         let result_2 = get_mcf_by_mcv(&client, &"olivia");
    //         let freq = result_2.await;

    //         match something {
    //             Ok(vec) => {
    //                 println!("Результат: {:#?} таблиц", vec.len());

    //                 compute_prob(vec);
    //             }
    //             Err(err) => {
    //                 eprintln!("Подробная ошибка БД: {}", err);
    //             }
    //         }
    //         match freq {
    //             Ok(f) => {
    //                 println!("The freq is {}\n", f)
    //             }
    //             Err(err) => {
    //                 eprintln!("Подробная ошибка БД: {}", err);
    //             }
    //         }
    //     }
    //     Err(err) => {
    //         eprintln!("Подробная ошибка БД: {}", err);
    //     }
    // }

    //     Ok(())
}
