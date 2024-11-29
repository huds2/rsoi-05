use std::{error::Error, collections::HashMap};
use crate::{arc, server::{router, Services}};
use async_trait::async_trait;
use requester::{Response, Requester};
use warp::{Filter, reply::Reply, reject::Rejection};


#[derive(Clone)]
struct MockRequester {
    current_request: usize,
    responses: Vec<Response>
}

impl MockRequester {
    pub fn new(responses: Vec<Response>) -> Self {
        MockRequester {
            current_request: 0,
            responses
        }
    }
}

#[async_trait]
impl Requester for MockRequester {
    #[allow(unused_variables)]
    async fn send(&mut self,
                  url:String,
                  method:requester::RequestMethod,
                  headers:std::collections::HashMap<String,String>,
                  body:String) -> Result<Response, Box<dyn Error>> {
        if self.current_request >= self.responses.len() {
            panic!("Ran out of respones");
        }
        self.current_request += 1;
        Ok(self.responses[self.current_request - 1].clone())
    }
}

fn create_router(responses: Vec<Response>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let services = Services {
        flights: "".to_owned(),
        tickets: "".to_owned(),
        bonuses: "".to_owned(),
        requester: Box::new(MockRequester::new(responses))
    };
    router("api/v1", arc!(services))
}


#[tokio::test]
async fn health_check() {
    let router = create_router(vec![
        Response {
            code: 200,
            body: "".to_owned(),
            header: HashMap::new()
        },
        Response {
            code: 500,
            body: "".to_owned(),
            header: HashMap::new()
        },
        Response {
            code: 200,
            body: "".to_owned(),
            header: HashMap::new()
        }
    ]);
    let res = warp::test::request()
        .method("GET")
        .path("/manage/health")
        .reply(&router).await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.body(), "{\"gateway\":true,\"flights\":true,\"tickets\":true,\"bonuses\":false}");
}

#[tokio::test]
async fn get_flights() {
    let router = create_router(vec![
        Response {
            code: 200,
            body: "{\n  \"page\": 1,\n  \"pageSize\": 1,\n  \"totalElements\": 1,\n  \"items\": [\n    {\"flightNumber\": \"AFL031\",\n      \"fromAirport\": \"Sheremetevo\",\n      \"toAirport\": \"Pulkovo\",\n      \"date\": \"2021-10-08 20:00\",\n      \"price\": 1500\n    }\n  ]\n}".to_owned(),
            header: HashMap::new()
        }
    ]);
    let res = warp::test::request()
        .method("GET")
        .path("/api/v1/flights")
        .reply(&router).await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.body().to_owned(), "{\"page\":1,\"pageSize\":1,\"totalElements\":1,\"items\":[{\"flightNumber\":\"AFL031\",\"fromAirport\":\"Sheremetevo\",\"toAirport\":\"Pulkovo\",\"date\":\"2021-10-08 20:00\",\"price\":1500}]}");
}
