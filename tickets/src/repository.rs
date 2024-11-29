use std::error::Error;
use async_trait::async_trait;
use tokio_postgres::{Client, NoTls};
use crate::TicketError;
use structs::{Ticket, TicketPost};
use uuid::Uuid;

#[async_trait]
pub trait TicketRepository: Sync + Send {
    async fn init(&mut self) ->  Result<(), Box<dyn Error>>;
    async fn list(&mut self) ->  Result<Vec<Ticket>, Box<dyn Error>>;
    async fn get(&mut self, uuid: Uuid) ->  Result<Ticket, Box<dyn Error>>;
    async fn create(&mut self, ticket: TicketPost, username: String) ->  Result<Uuid, Box<dyn Error>>;
    async fn cancel(&mut self, uuid: Uuid) ->  Result<(), Box<dyn Error>>;
    async fn delete(&mut self, uuid: Uuid) ->  Result<(), Box<dyn Error>>;
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
impl TicketRepository for Repository {
    async fn init(&mut self) ->  Result<(), Box<dyn Error>> {
        self.client.execute("
            CREATE TABLE IF NOT EXISTS ticket
            (
                id            SERIAL PRIMARY KEY,
                ticket_uid    uuid UNIQUE NOT NULL,
                username      VARCHAR(80) NOT NULL,
                flight_number VARCHAR(20) NOT NULL,
                price         INT         NOT NULL,
                status        VARCHAR(20) NOT NULL CHECK (status IN ('PAID', 'CANCELED'))
            );
        ", &[]).await?;
        Ok(())
    }
    async fn list(&mut self) ->  Result<Vec<Ticket>, Box<dyn Error>> {
        let mut list = vec![];
        for row in self.client.query("
            SELECT * FROM ticket
        ", &[]).await? {
            list.push(Ticket {
                id: row.get(0),
                ticket_uid: row.get(1),
                username: row.get(2),
                flight_number: row.get(3),
                price: row.get(4),
                status: row.get(5)
            })
        }
        Ok(list)
    }
    async fn get(&mut self, uuid: Uuid) ->  Result<Ticket, Box<dyn Error>> {
        for row in self.client.query(&format!("
        SELECT * FROM ticket WHERE ticket_uid = '{}'
        ", uuid), &[]).await? {
            return Ok(Ticket {
                id: row.get(0),
                ticket_uid: row.get(1),
                username: row.get(2),
                flight_number: row.get(3),
                price: row.get(4),
                status: row.get(5)
            })
        }
        Err(TicketError::NotFoundError.into())
    }
    async fn create(&mut self, ticket: TicketPost, username: String) ->  Result<Uuid, Box<dyn Error>> {
        let ticket_uid = Uuid::new_v4();
        self.client.batch_execute(&format!("
            INSERT INTO ticket(ticket_uid, username, flight_number, price, status) VALUES
                ('{}', '{}', '{}', {}, '{}')
        ", ticket_uid.clone(), username, ticket.flight_number, ticket.price, "PAID")).await?;
        Ok(ticket_uid)
    }
    async fn cancel(&mut self, uuid: Uuid) ->  Result<(), Box<dyn Error>> {
        self.client.batch_execute(&format!("
            UPDATE ticket SET
                status = 'CANCELED'
            WHERE ticket_uid = '{}'
        ", uuid)).await?;
        Ok(())
    }
    async fn delete(&mut self, uuid: Uuid) ->  Result<(), Box<dyn Error>> {
        self.client.batch_execute(&format!("
            DELETE FROM ticket
            WHERE ticket_uid = '{}'
        ", uuid)).await?;
        Ok(())
    }
}
