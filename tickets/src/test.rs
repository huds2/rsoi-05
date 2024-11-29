use std::error::Error;
use crate::{arc, repository::TicketRepository, server::router};
use async_trait::async_trait;
use structs::{Ticket, TicketPost};
use uuid::Uuid;


struct MockRepository {
    tickets: (Vec<Ticket>, usize)
}

impl MockRepository {
    pub fn new(tickets: Vec<Ticket>) -> Self {
        MockRepository {
            tickets: (tickets, 0)
        }
    }
}

#[allow(unused_variables)]
#[async_trait]
impl TicketRepository for MockRepository {
    async fn init(&mut self) ->  Result<(), Box<dyn Error>> {
        Ok(())
    }
    async fn list(&mut self) ->  Result<Vec<Ticket>, Box<dyn Error>> {
        Ok(self.tickets.0.clone())
    }
    async fn get(&mut self, uuid: Uuid) ->  Result<Ticket, Box<dyn Error>> {
        self.tickets.1 += 1;
        Ok(self.tickets.0[self.tickets.1 - 1].clone())
    }
    async fn create(&mut self, ticket: TicketPost, username: String) ->  Result<Uuid, Box<dyn Error>> {
        Ok(uuid::uuid!("914619a4-ade7-43cb-b086-9e88ca35a728"))
    }
    async fn cancel(&mut self, uuid: Uuid) ->  Result<(), Box<dyn Error>> {
        todo!()
    }
}


#[tokio::test]
async fn list_tickets() {
    let repository = arc!(MockRepository::new(
            vec![
                Ticket {
                    id: 0,
                    ticket_uid: uuid::uuid!("17ea0b3b-9efb-4be1-8db5-81512fe77c88"),
                    username: "someone".to_owned(),
                    flight_number: "AFL31".to_owned(),
                    price: 50,
                    status: "PAID".to_owned()
                }
            ]
            ));
    let router = router(repository);
    let res = warp::test::request()
        .method("GET")
        .path("/tickets")
        .header("X-User-Name", "someone")
        .reply(&router).await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.body(), "[{\"id\":0,\"ticket_uid\":\"17ea0b3b-9efb-4be1-8db5-81512fe77c88\",\"username\":\"someone\",\"flight_number\":\"AFL31\",\"price\":50,\"status\":\"PAID\"}]");
}

#[tokio::test]
async fn post_tickets() {
    let repository = arc!(MockRepository::new(
            vec![
                Ticket {
                    id: 0,
                    ticket_uid: uuid::uuid!("17ea0b3b-9efb-4be1-8db5-81512fe77c88"),
                    username: "someone".to_owned(),
                    flight_number: "AFL31".to_owned(),
                    price: 50,
                    status: "PAID".to_owned()
                }
            ]
            ));
    let router = router(repository);
    let res = warp::test::request()
        .method("POST")
        .path("/tickets")
        .header("X-User-Name", "someone")
        .body(serde_json::to_string(&TicketPost{
            flight_number: "AFL31".to_owned(),
            price: 50
        }).unwrap())
        .reply(&router).await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.body(), "{\"id\":0,\"ticket_uid\":\"17ea0b3b-9efb-4be1-8db5-81512fe77c88\",\"username\":\"someone\",\"flight_number\":\"AFL31\",\"price\":50,\"status\":\"PAID\"}");
}
