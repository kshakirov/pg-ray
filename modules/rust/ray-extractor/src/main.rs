use tokio_postgres::{Error, NoTls};
#[tokio::main]
async fn main() -> Result<(),Error> {
    println!("Hello, world!");
    Ok(())
    
}
