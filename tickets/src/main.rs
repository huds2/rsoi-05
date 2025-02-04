mod repository;
mod server;
#[cfg(test)]
mod test;
use jwtchecker::JWTChecker;
use repository::*;
use server::*;

use std::error::Error;
use std::env;
use custom_error::custom_error;

custom_error!{pub TicketError
    NotFoundError                            = "Ticket was not found",
}

#[macro_export]
macro_rules! arc{
    ($a:expr)=>{
        {
            std::sync::Arc::new(tokio::sync::Mutex::new($a))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let connection_str = env::var("PSQL_CONNECTION")?;
    let port = env::var("SERVER_PORT")?.parse()?;
    let repository = arc!(Repository::new(&connection_str).await?);
    repository.lock().await.init().await?;
    run_server(repository, port, arc!(JWTChecker::new(&env::var("RSA_PUB")?))).await;
    Ok(())
}
