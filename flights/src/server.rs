use std::{convert::Infallible, sync::Arc, error::Error};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use warp::{reply::{self, Reply}, Filter, Rejection};

use crate::{Flight, FlightRepository, WebFlight, WebFlightPage};

pub type WebResult<T> = std::result::Result<T, Rejection>;

async fn flight_to_webflight(flight: Flight, flight_repository: Arc<Mutex<dyn FlightRepository>>) -> Result<WebFlight, Box<dyn Error>> {
    let mut repo = flight_repository.lock().await;
    let from_airport = repo.get_airport(flight.from_airport_id).await.unwrap();
    let to_airtport = repo.get_airport(flight.to_airport_id).await?;
    let from_airport_str = format!("{} {}", from_airport.city, from_airport.name);
    let to_airtport_str = format!("{} {}", to_airtport.city, to_airtport.name);
    let flight_date = flight.datetime.format("%Y-%m-%d %H:%M").to_string();
    Ok(WebFlight { 
        flightNumber: flight.flight_number,
        fromAirport: from_airport_str,
        toAirport: to_airtport_str,
        date: flight_date,
        price: flight.price 
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paging {
    pub page: usize,
    pub size: usize,
}

async fn list_handler(paging: Paging,
                      flight_repository: Arc<Mutex<dyn FlightRepository>>) -> WebResult<Box<dyn Reply>> {
    let Ok(flight_list) = flight_repository.lock().await.list().await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    let mut paged_webflights = vec![];
    for i in (paging.page - 1) * paging.size..flight_list.len().min(paging.page * paging.size) {
        let Ok(webflight) = flight_to_webflight(flight_list[i].clone(), flight_repository.clone()).await else {
            let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
            return Ok(Box::new(reply));
        };
        paged_webflights.push(webflight);
    }
    return Ok(Box::new(reply::json(&WebFlightPage {
        page: paging.page,
        pageSize: paging.size,
        totalElements: flight_list.len(),
        items: paged_webflights
    })));
}

async fn get_handler(id: String,
                     flight_repository: Arc<Mutex<dyn FlightRepository>>) -> WebResult<Box<dyn Reply>> {
    let Ok(flight) = flight_repository.lock().await.get_flight(id).await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    let Ok(flight) = flight_to_webflight(flight, flight_repository).await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    return Ok(Box::new(reply::json(&flight)));
}

async fn health_check_handler() -> WebResult<impl Reply> {
    return Ok(warp::reply::with_status("Up and running", warp::http::StatusCode::OK))
}

fn with_arc<T: Send + ?Sized>(arc: Arc<Mutex<T>>) -> impl Filter<Extract = (Arc<Mutex<T>>,), Error = Infallible> + Clone {
    warp::any().map(move || arc.clone())
}

pub fn router(repository: Arc<Mutex<dyn FlightRepository>>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let log = warp::log::custom(|info| {
        eprintln!(
            "{} {} {}",
            info.method(),
            info.path(),
            info.status(),
        );
    });
    let list_route = warp::path!("flights")
        .and(warp::get())
        .and(warp::query::<Paging>())
        .and(with_arc(repository.clone()))
        .and_then(list_handler);
    let get_route = warp::path!("flights" / String)
        .and(warp::get())
        .and(with_arc(repository.clone()))
        .and_then(get_handler);
    let health_route = warp::path!("manage" / "health")
        .and(warp::get())
        .and_then(health_check_handler);
    let routes = get_route
        .or(list_route)
        .or(health_route)
        .with(log);
    routes
}

pub async fn run_server(repository: Arc<Mutex<dyn FlightRepository>>, port: u16) {
    let router = router(repository);
    warp::serve(router)
        .run(([0, 0, 0, 0], port))
        .await
}
