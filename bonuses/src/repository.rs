use std::error::Error;
use async_trait::async_trait;
use tokio_postgres::{Client, NoTls};
use crate::PrivilegeError;
use structs::{Privilege,  PrivilegeHistory, PrivilegeHistoryPost};
use uuid::Uuid;

#[async_trait]
pub trait PrivilegeRepository: Sync + Send {
    async fn init(&mut self) ->  Result<(), Box<dyn Error>>;
    async fn get_privilege(&mut self, username: String) ->  Result<Privilege, Box<dyn Error>>;
    async fn get_privilege_history(&mut self, username: String) ->  Result<Vec<PrivilegeHistory>, Box<dyn Error>>;
    async fn get_privilege_history_by_ticket(&mut self, ticket_uid: Uuid) ->  Result<Vec<PrivilegeHistory>, Box<dyn Error>>;
    async fn add_history(&mut self, data: PrivilegeHistoryPost) ->  Result<(), Box<dyn Error>>;
    async fn update_balance(&mut self, username: String, difference: i32) ->  Result<(), Box<dyn Error>>;
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
impl PrivilegeRepository for Repository {
    async fn init(&mut self) ->  Result<(), Box<dyn Error>> {
        self.client.execute("
            CREATE TABLE IF NOT EXISTS privilege
            (
                id       SERIAL PRIMARY KEY,
                username VARCHAR(80) NOT NULL UNIQUE,
                status   VARCHAR(80) NOT NULL DEFAULT 'BRONZE'
                    CHECK (status IN ('BRONZE', 'SILVER', 'GOLD')),
                balance  INT
            );
        ", &[]).await?;
        self.client.execute("
            CREATE TABLE IF NOT EXISTS privilege_history
            (
                id             SERIAL PRIMARY KEY,
                privilege_id   INT REFERENCES privilege (id),
                ticket_uid     uuid        NOT NULL,
                datetime       TIMESTAMP   NOT NULL,
                balance_diff   INT         NOT NULL,
                operation_type VARCHAR(20) NOT NULL
                    CHECK (operation_type IN ('FILL_IN_BALANCE', 'DEBIT_THE_ACCOUNT'))
            );
        ", &[]).await?;
        Ok(())
    }
    async fn get_privilege(&mut self, username: String) ->  Result<Privilege, Box<dyn Error>> {
        println!("searching for '{}'", username);
        for row in self.client.query(&format!("
        SELECT * FROM privilege WHERE username = '{}'
        ", username), &[]).await? {
            return Ok(Privilege {
                id: row.get(0),
                username: row.get(1),
                status: row.get(2),
                balance: row.get(3)
            })
        }
        println!("Couldnt find any");
        Err(PrivilegeError::NotFoundError.into())
    }
    async fn get_privilege_history(&mut self, username: String) ->  Result<Vec<PrivilegeHistory>, Box<dyn Error>> {
        let privilege_id = self.get_privilege(username).await?.id;
        let mut list = vec![];
        for row in self.client.query(&format!("
            SELECT * FROM privilege_history WHERE privilege_id = {}
        ", privilege_id), &[]).await? {
            list.push(PrivilegeHistory {
                id: row.get(0),
                privilege_id: row.get(1),
                ticket_uid: row.get(2),
                datetime: row.get(3),
                balance_diff: row.get(4),
                operation_type: row.get(5)
            })
        }
        Ok(list)
    }
    async fn get_privilege_history_by_ticket(&mut self, ticket_uid: Uuid) ->  Result<Vec<PrivilegeHistory>, Box<dyn Error>> {
        let mut list = vec![];
        for row in self.client.query(&format!("
            SELECT * FROM privilege_history WHERE ticket_uid = '{}'
        ", ticket_uid), &[]).await? {
            list.push(PrivilegeHistory {
                id: row.get(0),
                privilege_id: row.get(1),
                ticket_uid: row.get(2),
                datetime: row.get(3),
                balance_diff: row.get(4),
                operation_type: row.get(5)
            })
        }
        Ok(list)
    }
    async fn add_history(&mut self, data: PrivilegeHistoryPost) ->  Result<(), Box<dyn Error>> {
        let privilege_id = self.get_privilege(data.username.clone()).await?.id;
        let datetime = chrono::offset::Utc::now().naive_local();
        self.client.batch_execute(&format!("
            INSERT INTO privilege_history(privilege_id, ticket_uid, datetime, balance_diff, operation_type) VALUES
                ('{}', '{}', '{}', {}, '{}')
        ", privilege_id, data.ticket_uid, datetime, data.balance_diff, data.operation_type)).await?;
        if data.operation_type == "FILL_IN_BALANCE" {
            self.update_balance(data.username, data.balance_diff).await?;
        }
        else {
            self.update_balance(data.username, -data.balance_diff).await?;
        }
        Ok(())
    }
    async fn update_balance(&mut self, username: String, difference: i32) ->  Result<(), Box<dyn Error>> {
        let balance = self.get_privilege(username.clone()).await?.balance;
        let new_balance = (balance + difference).max(0);
        self.client.batch_execute(&format!("
            UPDATE privilege SET
                balance = {}
            WHERE username = '{}'
        ", new_balance, username)).await?;
        Ok(())
    }
}
