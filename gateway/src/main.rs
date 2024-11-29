use std::error::Error;
use custom_error::custom_error;
use std::env;
use jwtchecker::JWTChecker;

use requester::Reqwester;
mod server;
#[cfg(test)]
mod test;
use server::*;

custom_error!{pub GatewayError
    TicketNotFoundError                             = "Ticket was not found",
    FlightNotFoundError                             = "Flight was not found",
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
    let port = env::var("SERVER_PORT")?.parse()?;
    run_server("api/v1", port, arc!(Services { 
        flights: env::var("FLIGHTS_URL")?.to_owned(),
        tickets: env::var("TICKETS_URL")?.to_owned(),
        bonuses: env::var("BONUSES_URL")?.to_owned(),
        requester: Box::new(Reqwester {}),
        queue: vec![],
        checker: JWTChecker::new(&env::var("RSA_PUB")?),
    })).await;
    Ok(())
}
