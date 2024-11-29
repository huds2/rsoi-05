use std::error::Error;
use crate::{arc, repository::FlightRepository, server::router};
use async_trait::async_trait;
use chrono::{Utc, TimeZone};
use structs::{Flight, Airport};


struct MockRepository {
    flights: Option<Vec<Flight>>,
    airport: Option<Airport>
}

impl MockRepository {
    pub fn new(flights: Option<Vec<Flight>>, airport: Option<Airport>) -> Self {
        MockRepository {
            flights,
            airport
        }
    }
}

#[allow(unused_variables)]
#[async_trait]
impl FlightRepository for MockRepository {
    async fn init(&mut self) ->  Result<(), Box<dyn Error>> {
        Ok(())
    }
    async fn list(&mut self) ->  Result<Vec<Flight>, Box<dyn Error>> {
        Ok(self.flights.clone().unwrap())
    }
    async fn get_flight(&mut self, flight_number: String) -> Result<Flight, Box<dyn Error>> {
        Ok(self.flights.clone().unwrap()[0].clone())
    }
    async fn get_airport(&mut self, airport_id: i32) -> Result<Airport, Box<dyn Error>> {
        Ok(self.airport.clone().unwrap())
    }
}


#[tokio::test]
async fn get_flight() {
    let repository = arc!(MockRepository::new(
            Some(vec![
                 Flight {
                     id: 1,
                     flight_number: "AFL31".to_owned(),
                     datetime: Utc.timestamp_opt(1589717600, 0).unwrap(),
                     from_airport_id: 1,
                     to_airport_id: 2,
                     price: 1500
                 }
            ]),
            Some(Airport {
                id: 1,
                name: "Airport".to_owned(),
                city: "City".to_owned(),
                country: Some("Country".to_owned())
            })));
    let router = router(repository);
    let res = warp::test::request()
        .method("GET")
        .path("/flights?page=1&size=5")
        .reply(&router).await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.body(), "{\"page\":1,\"pageSize\":5,\"totalElements\":1,\"items\":[{\"flightNumber\":\"AFL31\",\"fromAirport\":\"City Airport\",\"toAirport\":\"City Airport\",\"date\":\"2020-05-17 12:13\",\"price\":1500}]}");
}
