use std::{convert::Infallible, sync::Arc};
use jwtchecker::JWTChecker;
use tokio::sync::Mutex;
use uuid::Uuid;
use warp::{reply::{self, Reply}, Filter, Rejection};
use structs::TicketPost;

use super::TicketRepository;

pub type WebResult<T> = std::result::Result<T, Rejection>;

async fn list_handler(auth_token: String,
                      ticket_repository: Arc<Mutex<dyn TicketRepository>>,
                      checker: Arc<Mutex<JWTChecker>>) -> WebResult<Box<dyn Reply>> {
    let jwtchecker = checker.lock().await.clone();
    let Ok(username) = jwtchecker.decode_header(&auth_token) else {
        return Ok(Box::new(warp::reply::with_status("Not authorized", warp::http::StatusCode::UNAUTHORIZED)))
    };

    let Ok(mut tickets) = ticket_repository.lock().await.list().await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    tickets.retain(|x| x.username == username);
    return Ok(Box::new(reply::json(&tickets)));
}

async fn get_handler(id: Uuid,
                     auth_token: String,
                     ticket_repository: Arc<Mutex<dyn TicketRepository>>,
                     checker: Arc<Mutex<JWTChecker>>) -> WebResult<Box<dyn Reply>> {
    let jwtchecker = checker.lock().await.clone();
    let Ok(username) = jwtchecker.decode_header(&auth_token) else {
        return Ok(Box::new(warp::reply::with_status("Not authorized", warp::http::StatusCode::UNAUTHORIZED)))
    };

    let Ok(ticket) = ticket_repository.lock().await.get(id).await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    if ticket.username == username {
        return Ok(Box::new(reply::json(&ticket)));
    }
    let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
    return Ok(Box::new(reply));
}

async fn post_handler(body: TicketPost,
                      auth_token: String,
                      ticket_repository: Arc<Mutex<dyn TicketRepository>>,
                      checker: Arc<Mutex<JWTChecker>>) -> WebResult<Box<dyn Reply>> {
    let jwtchecker = checker.lock().await.clone();
    let Ok(username) = jwtchecker.decode_header(&auth_token) else {
        return Ok(Box::new(warp::reply::with_status("Not authorized", warp::http::StatusCode::UNAUTHORIZED)))
    };

    let Ok(id) = ticket_repository.lock().await.create(body, username).await else {
        return Ok(Box::new(warp::reply::with_status("Encountered an error", warp::http::StatusCode::INTERNAL_SERVER_ERROR)));
    };
    let Ok(ticket) = ticket_repository.lock().await.get(id).await else {
        return Ok(Box::new(warp::reply::with_status("Encountered an error", warp::http::StatusCode::INTERNAL_SERVER_ERROR)));
    };
    return Ok(Box::new(reply::json(&ticket)));
}

async fn cancel_handler(id: Uuid,
                        auth_token: String,
                        ticket_repository: Arc<Mutex<dyn TicketRepository>>,
                        checker: Arc<Mutex<JWTChecker>>) -> WebResult<Box<dyn Reply>> {
    let jwtchecker = checker.lock().await.clone();
    let Ok(username) = jwtchecker.decode_header(&auth_token) else {
        return Ok(Box::new(warp::reply::with_status("Not authorized", warp::http::StatusCode::UNAUTHORIZED)))
    };

    let not_found_reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
    match ticket_repository.lock().await.get(id).await {
        Ok(ticket) => {
            if ticket.username != username {
                return Ok(Box::new(not_found_reply));
            }
        },
        Err(_) => {
            return Ok(Box::new(not_found_reply));
        }
    }
    match ticket_repository.lock().await.cancel(id).await {
        Ok(_) => {
            return Ok(Box::new(warp::reply::with_status("Deleted ticket", warp::http::StatusCode::NO_CONTENT)))
        },
        _ => {}
    }
    return Ok(Box::new(not_found_reply));
}

async fn delete_handler(id: Uuid,
                        auth_token: String,
                        ticket_repository: Arc<Mutex<dyn TicketRepository>>,
                        checker: Arc<Mutex<JWTChecker>>) -> WebResult<Box<dyn Reply>> {
    let jwtchecker = checker.lock().await.clone();
    let Ok(username) = jwtchecker.decode_header(&auth_token) else {
        return Ok(Box::new(warp::reply::with_status("Not authorized", warp::http::StatusCode::UNAUTHORIZED)))
    };

    let not_found_reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
    match ticket_repository.lock().await.get(id).await {
        Ok(ticket) => {
            if ticket.username != username {
                return Ok(Box::new(not_found_reply));
            }
        },
        Err(_) => {
            return Ok(Box::new(not_found_reply));
        }
    }
    match ticket_repository.lock().await.delete(id).await {
        Ok(_) => {
            return Ok(Box::new(warp::reply::with_status("Deleted ticket", warp::http::StatusCode::NO_CONTENT)))
        },
        _ => {}
    }
    return Ok(Box::new(not_found_reply));
}

async fn health_check_handler() -> WebResult<impl Reply> {
    return Ok(warp::reply::with_status("Up and running", warp::http::StatusCode::OK))
}

fn with_arc<T: Send + ?Sized>(arc: Arc<Mutex<T>>) -> impl Filter<Extract = (Arc<Mutex<T>>,), Error = Infallible> + Clone {
    warp::any().map(move || arc.clone())
}

pub fn router(repository: Arc<Mutex<dyn TicketRepository>>, checker: Arc<Mutex<jwtchecker::JWTChecker>>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let log = warp::log::custom(|info| {
        eprintln!(
            "{} {} {}",
            info.method(),
            info.path(),
            info.status(),
        );
    });
    let list_route = warp::path!("tickets")
        .and(warp::get())
        .and(warp::header("Authorization"))
        .and(with_arc(repository.clone()))
        .and(with_arc(checker.clone()))
        .and_then(list_handler);
    let create_route = warp::path!("tickets")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::header("Authorization"))
        .and(with_arc(repository.clone()))
        .and(with_arc(checker.clone()))
        .and_then(post_handler);
    let get_route = warp::path!("tickets" / Uuid)
        .and(warp::get())
        .and(warp::header("Authorization"))
        .and(with_arc(repository.clone()))
        .and(with_arc(checker.clone()))
        .and_then(get_handler);
    let cancel_route = warp::path!("tickets" / Uuid / "cancel")
        .and(warp::delete())
        .and(warp::header("Authorization"))
        .and(with_arc(repository.clone()))
        .and(with_arc(checker.clone()))
        .and_then(cancel_handler);
    let delete_route = warp::path!("tickets" / Uuid)
        .and(warp::delete())
        .and(warp::header("Authorization"))
        .and(with_arc(repository.clone()))
        .and(with_arc(checker.clone()))
        .and_then(delete_handler);
    let health_route = warp::path!("manage" / "health")
        .and(warp::get())
        .and_then(health_check_handler);
    let routes = create_route
        .or(get_route)
        .or(list_route)
        .or(cancel_route)
        .or(delete_route)
        .or(health_route)
        .with(log);
    routes
}

pub async fn run_server(repository: Arc<Mutex<dyn TicketRepository>>, port: u16, checker: Arc<Mutex<JWTChecker>>) {
    let router = router(repository, checker);
    warp::serve(router)
        .run(([0, 0, 0, 0], port))
        .await
}
