use std::{collections::HashMap, convert::Infallible, sync::Arc, error::Error};
use serde::{Deserialize, Serialize};
use tokio::{sync::Mutex};
use uuid::Uuid;
use warp::{reply::{self, Reply}, Filter, Rejection};
use requester::{send_typed, RequestMethod, Requester, RequesterError};
use structs::{Balance, CombinedPurchaseResponse, HealthCheckResponse, PrivilegeGet, PurchasePost, PurchaseResponse, Ticket, TicketPost, TicketPostBalance, TicketResponse, User, WebFlight, WebFlightPage};
use jwtchecker::JWTChecker;

pub type WebResult<T> = std::result::Result<T, Rejection>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paging {
    pub page: Option<usize>,
    pub size: Option<usize>,
}

async fn list_flights_handler(auth_token: String,
                              paging: Paging,
                              services: Arc<Mutex<Services>>) -> WebResult<Box<dyn Reply>> {
    let jwtchecker = services.lock().await.checker.clone();
    if jwtchecker.decode_header(&auth_token).is_err() {
        return Ok(Box::new(warp::reply::with_status("Not authorized", warp::http::StatusCode::UNAUTHORIZED)))
    }

    let flight_url = services.lock().await.flights.clone();
    let mut requester = &mut services.lock().await.requester.clone();
    let Ok(flights) = send_typed::<WebFlightPage>(
        &mut requester,
        format!("{}?page={}&size={}", flight_url, paging.page.unwrap_or(1), paging.size.unwrap_or(10)),
        RequestMethod::GET,
        HashMap::new(),
        "".to_string()).await else {
        let reply = warp::reply::with_status("Internal error", warp::http::StatusCode::INTERNAL_SERVER_ERROR);
        return Ok(Box::new(reply));
    };
    return Ok(Box::new(reply::json(&flights)));
}

async fn ticket_to_responseticket(ticket: Ticket, 
                                  services: Arc<Mutex<Services>>) -> Result<TicketResponse, Box<dyn Error>> {
    let flights_url = services.lock().await.flights.clone();
    let mut requester = &mut services.lock().await.requester.clone();
    let flight = match send_typed::<WebFlight>(
        &mut requester,
        format!("{}/{}", flights_url, ticket.flight_number),
        RequestMethod::GET,
        HashMap::new(),
        "".to_string()).await {
        Ok(val) => val,
        Err(e) => {
            if e.is::<RequesterError>() {
                return Err(e);
            };
            return Ok(TicketResponse {
                date: "1970-01-01 00:00".to_owned(),
                ticketUid: ticket.ticket_uid,
                flightNumber: ticket.flight_number,
                fromAirport: "Departure airport".to_owned(),
                toAirport: "Destination airport".to_owned(),
                status: ticket.status,
                price: ticket.price
            });
        }
    };
    Ok(TicketResponse {
        date: flight.date,
        ticketUid: ticket.ticket_uid,
        flightNumber: flight.flightNumber,
        fromAirport: flight.fromAirport,
        toAirport: flight.toAirport,
        price: flight.price,
        status: ticket.status
    })
}

async fn list_tickets_handler(auth_token: String,
                              services: Arc<Mutex<Services>>) -> WebResult<Box<dyn Reply>> {
    let jwtchecker = services.lock().await.checker.clone();
    if jwtchecker.decode_header(&auth_token).is_err() {
        return Ok(Box::new(warp::reply::with_status("Not authorized", warp::http::StatusCode::UNAUTHORIZED)))
    }

    let ticket_url = services.lock().await.tickets.clone();
    let mut requester = &mut services.lock().await.requester.clone();
    let tickets = match send_typed::<Vec<Ticket>>(
        &mut requester,
        ticket_url,
        RequestMethod::GET,
        HashMap::from([
            ("Authorization".to_owned(), auth_token)
        ]),
        "".to_string()).await {
        Ok(val) => val,
        Err(e) => {
            let reply = if e.is::<RequesterError>() {
                warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND)
            }
            else {
                warp::reply::with_status("Internal error", warp::http::StatusCode::INTERNAL_SERVER_ERROR)
            };
            return Ok(Box::new(reply));
        }
    };
    let mut response_tickets = vec![];
    for ticket in tickets {
        let Ok(ticket) = ticket_to_responseticket(ticket, services.clone()).await else {
            let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
            return Ok(Box::new(reply));
        };
        response_tickets.push(ticket);
    }
    return Ok(Box::new(reply::json(&response_tickets)));
}

async fn get_ticket_handler(ticket_uid: Uuid,
                            auth_token: String,
                            services: Arc<Mutex<Services>>) -> WebResult<Box<dyn Reply>> {
    let jwtchecker = services.lock().await.checker.clone();
    if jwtchecker.decode_header(&auth_token).is_err() {
        return Ok(Box::new(warp::reply::with_status("Not authorized", warp::http::StatusCode::UNAUTHORIZED)))
    }

    let ticket_url = services.lock().await.tickets.clone();
    let mut requester = &mut services.lock().await.requester.clone();
    let ticket = match send_typed::<Ticket>(
        &mut requester,
        format!("{}/{}", ticket_url, ticket_uid),
        RequestMethod::GET,
        HashMap::from([
            ("Authorization".to_owned(), auth_token)
        ]),
        "".to_string()).await {
        Ok(val) => val,
        Err(e) => {
            let reply = if e.is::<RequesterError>() {
                warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND)
            }
            else {
                warp::reply::with_status("Internal error", warp::http::StatusCode::INTERNAL_SERVER_ERROR)
            };
            return Ok(Box::new(reply));
        }
    };
    let Ok(ticket) = ticket_to_responseticket(ticket, services.clone()).await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    return Ok(Box::new(reply::json(&ticket)));
}

async fn get_privilege_handler(auth_token: String,
                               services: Arc<Mutex<Services>>) -> WebResult<Box<dyn Reply>> {
    let jwtchecker = services.lock().await.checker.clone();
    if jwtchecker.decode_header(&auth_token).is_err() {
        return Ok(Box::new(warp::reply::with_status("Not authorized", warp::http::StatusCode::UNAUTHORIZED)))
    }

    let privilege_url = services.lock().await.bonuses.clone();
    let mut requester = &mut services.lock().await.requester.clone();
    let privilege = match send_typed::<PrivilegeGet>(
        &mut requester,
        privilege_url,
        RequestMethod::GET,
        HashMap::from([
            ("Authorization".to_owned(), auth_token)
        ]),
        "".to_string()).await {
        Ok(val) => val,
        Err(e) => {
            let reply = if e.is::<RequesterError>() {
                warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND)
            }
            else {
                warp::reply::with_status("Internal error", warp::http::StatusCode::INTERNAL_SERVER_ERROR)
            };
            return Ok(Box::new(reply));
        }
    };
    return Ok(Box::new(reply::json(&privilege)));
}

async fn get_user_handler(auth_token: String,
                               services: Arc<Mutex<Services>>) -> WebResult<Box<dyn Reply>> {
    let jwtchecker = services.lock().await.checker.clone();
    if jwtchecker.decode_header(&auth_token).is_err() {
        return Ok(Box::new(warp::reply::with_status("Not authorized", warp::http::StatusCode::UNAUTHORIZED)))
    }

    let privilege_url = services.lock().await.bonuses.clone();
    let ticket_url = services.lock().await.tickets.clone();
    let mut requester = &mut services.lock().await.requester.clone();
    let privilege = match send_typed::<PrivilegeGet>(
        &mut requester,
        privilege_url,
        RequestMethod::GET,
        HashMap::from([
            ("Authorization".to_owned(), auth_token.clone())
        ]),
        "".to_string()).await {
        Ok(val) => val,
        Err(e) => {
            if e.is::<RequesterError>() {
                let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
                return Ok(Box::new(reply));
            }
            PrivilegeGet {
                balance: 0,
                status: "Generic status".to_owned(),
                history: vec![]
            }
        }
    };
    let tickets = match send_typed::<Vec<Ticket>>(
        &mut requester,
        ticket_url,
        RequestMethod::GET,
        HashMap::from([
            ("Authorization".to_owned(), auth_token)
        ]),
        "".to_string()).await {
        Ok(val) => val,
        Err(e) => {
            if e.is::<RequesterError>() {
                let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
                return Ok(Box::new(reply));
            }
            vec![]
        }
    };
    let mut response_tickets = vec![];
    for ticket in tickets {
        let Ok(ticket) = ticket_to_responseticket(ticket, services.clone()).await else {
            let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
            return Ok(Box::new(reply));
        };
        response_tickets.push(ticket);
    }
    return Ok(Box::new(reply::json(&User{
        tickets: response_tickets,
        privilege: Balance {
            balance: privilege.balance,
            status: privilege.status
        }
    })));
}

async fn post_ticket_handler(auth_token: String,
                             body: TicketPostBalance,
                             services: Arc<Mutex<Services>>) -> WebResult<Box<dyn Reply>> {
    let jwtchecker = services.lock().await.checker.clone();
    if jwtchecker.decode_header(&auth_token).is_err() {
        return Ok(Box::new(warp::reply::with_status("Not authorized", warp::http::StatusCode::UNAUTHORIZED)))
    }

    let flight_url = services.lock().await.flights.clone();
    let ticket_url = services.lock().await.tickets.clone();
    let privilege_url = services.lock().await.bonuses.clone();
    let requester = &mut services.lock().await.requester.clone();
    let ticket_post = TicketPost {
        flight_number: body.flightNumber,
        price: body.price
    };
    let flight = match send_typed::<WebFlight>(
        requester,
        format!("{}/{}", flight_url, ticket_post.flight_number),
        RequestMethod::GET,
        HashMap::new(),
        "".to_owned()).await {
        Ok(val) => val,
        Err(e) => {
            let reply = if e.is::<RequesterError>() {
                warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND)
            }
            else {
                warp::reply::with_status("Internal error", warp::http::StatusCode::INTERNAL_SERVER_ERROR)
            };
            return Ok(Box::new(reply));
        }
    };
    if flight.price != ticket_post.price {
        return Ok(Box::new(warp::reply::with_status("Ticket price does not match", warp::http::StatusCode::BAD_REQUEST)));
    }
    let ticket = match send_typed::<Ticket>(
        requester,
        ticket_url.clone(),
        RequestMethod::POST,
        HashMap::from([
            ("Authorization".to_owned(), auth_token.clone())
        ]),
        serde_json::to_string(&ticket_post).unwrap()).await {
        Ok(val) => val,
        Err(e) => {
            let reply = if e.is::<RequesterError>() {
                warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND)
            }
            else {
                warp::reply::with_status("Internal error", warp::http::StatusCode::INTERNAL_SERVER_ERROR)
            };
            return Ok(Box::new(reply));
        }
    };
    let privilege_post = PurchasePost {
        ticket_uid: ticket.ticket_uid,
        price: ticket.price,
        paid_from_balance: body.paidFromBalance
    };
    let Ok(purchase) = send_typed::<PurchaseResponse>(
        requester,
        privilege_url,
        RequestMethod::POST,
        HashMap::from([
            ("Authorization".to_owned(), auth_token.clone())
        ]),
        serde_json::to_string(&privilege_post).unwrap()).await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::INTERNAL_SERVER_ERROR);
        let Ok(response) = requester.send(
            format!("{}/{}", ticket_url, ticket.ticket_uid),
            RequestMethod::DELETE,
            HashMap::from([
                ("Authorization".to_owned(), auth_token.clone())
            ]),
            "".to_owned()).await else {
            let reply = warp::reply::with_status("Failed to return ticket", warp::http::StatusCode::INTERNAL_SERVER_ERROR);
            return Ok(Box::new(reply));
        };
        if response.code != 204 {
            let reply = warp::reply::with_status("Failed to return ticket", warp::http::StatusCode::INTERNAL_SERVER_ERROR);
            return Ok(Box::new(reply));
        }
        return Ok(Box::new(reply));
    };
    let Ok(ticket) = ticket_to_responseticket(ticket, services.clone()).await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    return Ok(Box::new(reply::json(&CombinedPurchaseResponse {
        ticketUid: ticket.ticketUid,
        flightNumber: ticket.flightNumber,
        fromAirport: ticket.fromAirport,
        toAirport: ticket.toAirport,
        date: ticket.date,
        price: ticket.price,
        paidByMoney: purchase.paid_by_money,
        paidByBonuses: purchase.paid_by_bonuses,
        status: ticket.status,
        privilege: Balance { 
            balance: purchase.balance,
            status: purchase.status 
        }
    })));
}

async fn delete_ticket_handler(ticket_uid: Uuid, 
                               auth_token: String,
                               services: Arc<Mutex<Services>>) -> WebResult<Box<dyn Reply>> {
    let jwtchecker = services.lock().await.checker.clone();
    if jwtchecker.decode_header(&auth_token).is_err() {
        return Ok(Box::new(warp::reply::with_status("Not authorized", warp::http::StatusCode::UNAUTHORIZED)))
    }

    let ticket_url = services.lock().await.tickets.clone();
    let mut requester = &mut services.lock().await.requester.clone();
    let ticket = match send_typed::<Ticket>(
        &mut requester,
        format!("{}/{}", ticket_url, ticket_uid),
        RequestMethod::GET,
        HashMap::from([
            ("Authorization".to_owned(), auth_token.clone())
        ]),
        "".to_string()).await {
        Ok(val) => val,
        Err(e) => {
            let reply = if e.is::<RequesterError>() {
                warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND)
            }
            else {
                warp::reply::with_status("Internal error", warp::http::StatusCode::INTERNAL_SERVER_ERROR)
            };
            return Ok(Box::new(reply));
        }
    };
    if ticket.status != "PAID" {
        let reply = warp::reply::with_status("Ticket already canceled", warp::http::StatusCode::BAD_REQUEST);
        return Ok(Box::new(reply));
    }
    let response = match requester.send(
        format!("{}/{}/cancel", ticket_url, ticket_uid),
        RequestMethod::DELETE,
        HashMap::from([
            ("Authorization".to_owned(), auth_token.clone())
        ]),
        "".to_string()).await {
        Ok(val) => val,
        Err(e) => {
            let reply = if e.is::<RequesterError>() {
                warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND)
            }
            else {
                warp::reply::with_status("Internal error", warp::http::StatusCode::INTERNAL_SERVER_ERROR)
            };
            return Ok(Box::new(reply));
        }
    };
    if response.code != 204 {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    }
    let queue = &mut services.lock().await.queue;
    queue.push(QueuedRequest { 
        ticket_uid,
        auth_token 
    });
    
    let reply = warp::reply::with_status("", warp::http::StatusCode::NO_CONTENT);
    return Ok(Box::new(reply));
}

async fn health_check_handler(services: Arc<Mutex<Services>>) -> WebResult<Box<dyn Reply>> {
    let ticket_url = services.lock().await.tickets.clone();
    let ticket_url = ticket_url[0..ticket_url.len() - 8].to_owned();
    let privilege_url = services.lock().await.bonuses.clone();
    let privilege_url = privilege_url[0..privilege_url.len() - 10].to_owned();
    let flight_url = services.lock().await.flights.clone();
    let flight_url = flight_url[0..flight_url.len() - 8].to_owned();
    let requester = &mut services.lock().await.requester.clone();
    let tickets = if let Ok(ticket_response) = requester.send(
        format!("{}/manage/health", ticket_url),
        RequestMethod::GET,
        HashMap::new(),
        "".to_owned()).await { 
        ticket_response.code == 200
    } else {
        false
    };
    let bonuses = if let Ok(privilege_response) = requester.send(
        format!("{}/manage/health", privilege_url),
        RequestMethod::GET,
        HashMap::new(),
        "".to_owned()).await { 
        privilege_response.code == 200
    } else {
        false
    };
    let flights = if let Ok(flight_response) = requester.send(
        format!("{}/manage/health", flight_url),
        RequestMethod::GET,
        HashMap::new(),
        "".to_owned()).await { 
        flight_response.code == 200
    } else {
        false
    };
    Ok(Box::new(reply::json(&HealthCheckResponse {
        gateway: true,
        flights,
        tickets,
        bonuses
    })))
}

async fn default_handler() -> WebResult<Box<dyn Reply>> {
    return Ok(Box::new(warp::reply::with_status("Not authorized", warp::http::StatusCode::UNAUTHORIZED)));
}

fn with_arc<T: Send + ?Sized>(arc: Arc<Mutex<T>>) -> impl Filter<Extract = (Arc<Mutex<T>>,), Error = Infallible> + Clone {
    warp::any().map(move || arc.clone())
}

#[derive(Clone)]
pub struct Services {
    pub flights: String,
    pub tickets: String,
    pub bonuses: String,
    pub requester: Box<dyn Requester>,
    pub queue: Vec<QueuedRequest>,
    pub checker: JWTChecker
}

#[derive(Clone)]
pub struct QueuedRequest {
    pub ticket_uid: Uuid,
    pub auth_token: String
}

pub fn router(root_url: &str, services: Arc<Mutex<Services>>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let log = warp::log::custom(|info| {
        eprintln!(
            "{} {} {}",
            info.method(),
            info.path(),
            info.status(),
        );
    });
    let list_flights_route = warp::path!("flights")
        .and(warp::get())
        .and(warp::header("Authorization"))
        .and(warp::query::<Paging>())
        .and(with_arc(services.clone()))
        .and_then(list_flights_handler);
    let list_tickets_route = warp::path!("tickets")
        .and(warp::get())
        .and(warp::header("Authorization"))
        .and(with_arc(services.clone()))
        .and_then(list_tickets_handler);
    let get_ticket_route = warp::path!("tickets" / Uuid)
        .and(warp::get())
        .and(warp::header("Authorization"))
        .and(with_arc(services.clone()))
        .and_then(get_ticket_handler);
    let get_privilege_route = warp::path!("privilege")
        .and(warp::get())
        .and(warp::header("Authorization"))
        .and(with_arc(services.clone()))
        .and_then(get_privilege_handler);
    let get_user_route = warp::path!("me")
        .and(warp::get())
        .and(warp::header("Authorization"))
        .and(with_arc(services.clone()))
        .and_then(get_user_handler);
    let post_ticket_route = warp::path!("tickets")
        .and(warp::post())
        .and(warp::header("Authorization"))
        .and(warp::body::json())
        .and(with_arc(services.clone()))
        .and_then(post_ticket_handler);
    let delete_ticket_route = warp::path!("tickets"/ Uuid)
        .and(warp::delete())
        .and(warp::header("Authorization"))
        .and(with_arc(services.clone()))
        .and_then(delete_ticket_handler);
    let health_route = warp::path!("manage" / "health")
        .and(warp::get())
        .and(with_arc(services.clone()))
        .and_then(health_check_handler);
    let default_route = warp::any()
        .and_then(default_handler);
    let routes = list_flights_route
        .or(list_tickets_route)
        .or(get_ticket_route)
        .or(get_privilege_route)
        .or(get_user_route)
        .or(post_ticket_route)
        .or(delete_ticket_route)
        .or(default_route);
    let mut root_route = warp::any().boxed();
    for segment in root_url.split("/") {
        root_route = root_route.and(warp::path(segment.to_owned())).boxed();
    }
    let routes = (root_route.and(routes)).or(health_route).with(log);

    tokio::task::spawn(async move {
        loop {
            {
                let mut requester;
                let privilege_url;
                {
                    let services_ptr = &services.lock().await;
                    requester = services_ptr.requester.clone();
                    privilege_url = services_ptr.bonuses.clone();
                }
                let queue = &mut services.lock().await.queue;
                let mut successful = true;
                while queue.len() > 0 && successful {
                    println!("Sending queued request");
                    let request = queue[0].clone();
                    if let Ok(_) = requester.send(
                        format!("{}?ticket_uid={}", privilege_url, request.ticket_uid),
                        RequestMethod::DELETE,
                        HashMap::from([
                            ("Authorization".to_owned(), request.auth_token)
                        ]),
                        "".to_owned()).await  {
                        queue.remove(0);
                    } else {
                        successful = false;
                    }
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    });

    routes
}

pub async fn run_server(root_url: &str, port: u16, services: Arc<Mutex<Services>>) {
    let router = router(root_url, services);
    warp::serve(router)
        .run(([0, 0, 0, 0], port))
        .await
}
