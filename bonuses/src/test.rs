use std::error::Error;
use crate::{arc, repository::PrivilegeRepository, server::router};
use async_trait::async_trait;
use structs::{Privilege, PrivilegeHistory, PrivilegeHistoryPost};
use uuid::Uuid;


struct MockRepository {
    privilege: Option<Privilege>,
    privilege_history: Option<Vec<PrivilegeHistory>>
}

impl MockRepository {
    pub fn new(privilege: Option<Privilege>, privilege_history: Option<Vec<PrivilegeHistory>>) -> Self {
        MockRepository {
            privilege,
            privilege_history
        }
    }
}

#[allow(unused_variables)]
#[async_trait]
impl PrivilegeRepository for MockRepository {
    async fn init(&mut self) ->  Result<(), Box<dyn Error>> {
        Ok(())
    }
    async fn get_privilege(&mut self, username: String) ->  Result<Privilege, Box<dyn Error>> {
        Ok(self.privilege.clone().unwrap())
    }
    async fn get_privilege_history(&mut self, username: String) ->  Result<Vec<PrivilegeHistory>, Box<dyn Error>> {
        Ok(self.privilege_history.clone().unwrap())
    }
    async fn get_privilege_history_by_ticket(&mut self, ticket_uid: Uuid) ->  Result<Vec<PrivilegeHistory>, Box<dyn Error>> {
        Ok(self.privilege_history.clone().unwrap())
    }
    async fn add_history(&mut self, data: PrivilegeHistoryPost) ->  Result<(), Box<dyn Error>> {
        todo!()
    }
    async fn update_balance(&mut self, username: String, difference: i32) ->  Result<(), Box<dyn Error>> {
        Ok(())
    }
}


#[tokio::test]
async fn get_privilege() {
    let repository = arc!(MockRepository::new(
            Some(Privilege {
                id: 0,
                username: "someone".to_owned(),
                status: "BRONZE".to_owned(),
                balance: 2000
            }), 
            Some(vec![])
            ));
    let router = router(repository);
    let res = warp::test::request()
        .method("GET")
        .path("/privilege")
        .header("X-User-Name", "someone")
        .reply(&router).await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.body(), "{\"balance\":2000,\"status\":\"BRONZE\",\"history\":[]}");
}
