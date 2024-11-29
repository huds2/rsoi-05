use std::error::Error;
use async_trait::async_trait;
use tokio_postgres::{Client, NoTls};
use structs::{Airport, Flight};
use crate::FlightError;

#[async_trait]
pub trait FlightRepository: Sync + Send {
    async fn init(&mut self) ->  Result<(), Box<dyn Error>>;
    async fn list(&mut self) ->  Result<Vec<Flight>, Box<dyn Error>>;
    async fn get_flight(&mut self, flight_number: String) -> Result<Flight, Box<dyn Error>>;
    async fn get_airport(&mut self, airport_id: i32) -> Result<Airport, Box<dyn Error>>;
}

pub struct Repository {
    client: Client
}

impl Repository {
    pub async fn new(connection_str: &str) -> Result<Self,  Box<dyn Error>> {
        let (client, connection) = tokio_postgres::connect(connection_str, NoTls).await?;
        tokio::spawn(async move {
            connection.await
        });
        Ok(Self { 
            client
        })
    }
}

#[async_trait]
impl FlightRepository for Repository {
    async fn init(&mut self) ->  Result<(), Box<dyn Error>> {
        self.client.execute("
            CREATE TABLE IF NOT EXISTS airport
            (
                id      SERIAL PRIMARY KEY,
                name    VARCHAR(255),
                city    VARCHAR(255),
                country VARCHAR(255)
            );
        ", &[]).await?;
        self.client.execute("
            CREATE TABLE IF NOT EXISTS flight
            (
                id              SERIAL PRIMARY KEY,
                flight_number   VARCHAR(20)              NOT NULL,
                datetime        TIMESTAMP WITH TIME ZONE NOT NULL,
                from_airport_id INT REFERENCES airport (id),
                to_airport_id   INT REFERENCES airport (id),
                price           INT                      NOT NULL
            );
        ", &[]).await?;
        Ok(())
    }
    async fn list(&mut self) ->  Result<Vec<Flight>, Box<dyn Error>> {
        let mut list = vec![];
        for row in self.client.query("
            SELECT * FROM flight
        ", &[]).await? {
            list.push(Flight {
                id: row.get(0),
                flight_number: row.get(1),
                datetime: row.get(2),
                from_airport_id: row.get(3),
                to_airport_id: row.get(4),
                price: row.get(5)
            })
        }
        Ok(list)
    }
    async fn get_flight(&mut self, flight_number: String) ->  Result<Flight, Box<dyn Error>> {
        for row in self.client.query(&format!("
        SELECT * FROM flight WHERE flight_number = '{}'
        ", flight_number), &[]).await? {
            return Ok(Flight {
                id: row.get(0),
                flight_number: row.get(1),
                datetime: row.get(2),
                from_airport_id: row.get(3),
                to_airport_id: row.get(4),
                price: row.get(5)
           })
        }
        Err(FlightError::NotFoundError.into())
    }
    async fn get_airport(&mut self, airport_id: i32) ->  Result<Airport, Box<dyn Error>> {
        for row in self.client.query(&format!("
        SELECT * FROM airport WHERE id = '{}'
        ", airport_id), &[]).await? {
            return Ok(Airport {
                id: row.get(0),
                name: row.get(1),
                city: row.get(2),
                country: row.get(3)
           })
        }
        Err(FlightError::NotFoundError.into())
    }
}
