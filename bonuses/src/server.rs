use std::{convert::Infallible, sync::Arc};
use jwtchecker::JWTChecker;
use tokio::sync::Mutex;
use uuid::Uuid;
use warp::{reply::{self, Reply}, Filter, Rejection};
use serde::{Serialize, Deserialize};
use crate::PrivilegeRepository;
use structs::{PrivilegeGet, PrivilegeHistory, PrivilegeHistoryGet, PurchasePost, PurchaseResponse, PrivilegeHistoryPost};

pub type WebResult<T> = std::result::Result<T, Rejection>;

fn privilege_history_to_web(history: &PrivilegeHistory) -> PrivilegeHistoryGet {
    PrivilegeHistoryGet { 
        date: history.datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        ticketUid: history.ticket_uid,
        balanceDiff: history.balance_diff,
        operationType: history.operation_type.clone()
    }
}

async fn get_handler(auth_token: String,
                     privilege_repository: Arc<Mutex<dyn PrivilegeRepository>>,
                     checker: Arc<Mutex<JWTChecker>>) -> WebResult<Box<dyn Reply>> {
    let jwtchecker = checker.lock().await.clone();
    let Ok(username) = jwtchecker.decode_header(&auth_token) else {
        return Ok(Box::new(warp::reply::with_status("Not authorized", warp::http::StatusCode::UNAUTHORIZED)))
    };

    println!("{}", username);

    let Ok(privilege) = privilege_repository.lock().await.get_privilege(username.clone()).await else {
        let reply = warp::reply::with_status("Could not find user", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    let Ok(privilege_history) = privilege_repository.lock().await.get_privilege_history(username).await else {
        let reply = warp::reply::with_status("Could not find history", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    let web_privilege_history = privilege_history.iter().map(|x| privilege_history_to_web(x)).collect();
    return Ok(Box::new(reply::json(&PrivilegeGet {
        balance: privilege.balance,
        status: privilege.status,
        history: web_privilege_history
    })));
}

async fn purchase_handler(auth_token: String,
                          body: PurchasePost,
                          privilege_repository: Arc<Mutex<dyn PrivilegeRepository>>,
                          checker: Arc<Mutex<JWTChecker>>) -> WebResult<Box<dyn Reply>> {
    let jwtchecker = checker.lock().await.clone();
    let Ok(username) = jwtchecker.decode_header(&auth_token) else {
        return Ok(Box::new(warp::reply::with_status("Not authorized", warp::http::StatusCode::UNAUTHORIZED)))
    };

    let mut paid_by_bonuses = 0;
    let paid_by_money;
    if body.paid_from_balance {
        let Ok(privilege) = privilege_repository.lock().await.get_privilege(username.clone()).await else {
            let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
            return Ok(Box::new(reply));
        };
        paid_by_bonuses = privilege.balance.min(body.price);
        paid_by_money = body.price - paid_by_bonuses;
        let Ok(_) = privilege_repository.lock().await.add_history(PrivilegeHistoryPost {
            username: username.clone(),
            ticket_uid: body.ticket_uid,
            balance_diff: paid_by_bonuses,
            operation_type: "DEBIT_THE_ACCOUNT".to_string()
        }).await else {
            let reply = warp::reply::with_status("Error", warp::http::StatusCode::INTERNAL_SERVER_ERROR);
            return Ok(Box::new(reply));
        };
    }
    else {
        paid_by_money = body.price;
        let Ok(_) = privilege_repository.lock().await.add_history(PrivilegeHistoryPost {
            username: username.clone(),
            ticket_uid: body.ticket_uid,
            balance_diff: (body.price as f64 * 0.1) as i32,
            operation_type: "FILL_IN_BALANCE".to_string()
        }).await else {
            let reply = warp::reply::with_status("Error", warp::http::StatusCode::INTERNAL_SERVER_ERROR);
            return Ok(Box::new(reply));
        };
    }
    let Ok(privilege) = privilege_repository.lock().await.get_privilege(username.clone()).await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };


    let reply = warp::reply::json(&PurchaseResponse {
        paid_by_money,
        paid_by_bonuses,
        balance: privilege.balance,
        status: privilege.status
    });
    return Ok(Box::new(reply));
}

#[derive(Serialize, Deserialize)]
struct RefundQuery {
    ticket_uid: Uuid,
}

async fn refund_handler(auth_token: String,
                        ticket_uid: RefundQuery,
                        privilege_repository: Arc<Mutex<dyn PrivilegeRepository>>,
                        checker: Arc<Mutex<JWTChecker>>) -> WebResult<Box<dyn Reply>> {
    let jwtchecker = checker.lock().await.clone();
    let Ok(username) = jwtchecker.decode_header(&auth_token) else {
        return Ok(Box::new(warp::reply::with_status("Not authorized", warp::http::StatusCode::UNAUTHORIZED)))
    };

    let Ok(privilege) = privilege_repository.lock().await.get_privilege(username.clone()).await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    let Ok(operations) = privilege_repository.lock().await.get_privilege_history_by_ticket(ticket_uid.ticket_uid).await else {
        let reply = warp::reply::with_status("Not found", warp::http::StatusCode::NOT_FOUND);
        return Ok(Box::new(reply));
    };
    for operation in operations {
        if operation.privilege_id == privilege.id {
            if operation.operation_type == "DEBIT_THE_ACCOUNT" {
                let Ok(_) = privilege_repository.lock().await.add_history(PrivilegeHistoryPost {
                    username: username.clone(),
                    ticket_uid: ticket_uid.ticket_uid,
                    balance_diff: operation.balance_diff,
                    operation_type: "FILL_IN_BALANCE".to_string()
                }).await else {
                    let reply = warp::reply::with_status("Error", warp::http::StatusCode::INTERNAL_SERVER_ERROR);
                    return Ok(Box::new(reply));
                };
            }
            else {
                let Ok(_) = privilege_repository.lock().await.add_history(PrivilegeHistoryPost {
                    username: username.clone(),
                    ticket_uid: ticket_uid.ticket_uid,
                    balance_diff: operation.balance_diff,
                    operation_type: "DEBIT_THE_ACCOUNT".to_string()
                }).await else {
                    let reply = warp::reply::with_status("Error", warp::http::StatusCode::INTERNAL_SERVER_ERROR);
                    return Ok(Box::new(reply));
                };
            }
        }
        else {
            let reply = warp::reply::with_status("Error", warp::http::StatusCode::FORBIDDEN);
            return Ok(Box::new(reply));
        }
    }

    let reply = warp::reply::with_status("Ticket refunded", warp::http::StatusCode::NO_CONTENT);
    return Ok(Box::new(reply));
}

async fn health_check_handler() -> WebResult<impl Reply> {
    return Ok(warp::reply::with_status("Up and running", warp::http::StatusCode::OK))
}

fn with_arc<T: Send + ?Sized>(arc: Arc<Mutex<T>>) -> impl Filter<Extract = (Arc<Mutex<T>>,), Error = Infallible> + Clone {
    warp::any().map(move || arc.clone())
}

pub fn router(repository: Arc<Mutex<dyn PrivilegeRepository>>, checker: Arc<Mutex<jwtchecker::JWTChecker>>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let log = warp::log::custom(|info| {
        eprintln!(
            "{} {} {}",
            info.method(),
            info.path(),
            info.status(),
        );
        for header in info.request_headers() {
            eprintln!(
                "   {} {:?}",
                header.0,
                header.1
            );
        }
    });
    let get_route = warp::path!("privilege")
        .and(warp::get())
        .and(warp::header("Authorization"))
        .and(with_arc(repository.clone()))
        .and(with_arc(checker.clone()))
        .and_then(get_handler);
    let purchase_route = warp::path!("privilege")
        .and(warp::post())
        .and(warp::header("Authorization"))
        .and(warp::body::json())
        .and(with_arc(repository.clone()))
        .and(with_arc(checker.clone()))
        .and_then(purchase_handler);
    let refund_route = warp::path!("privilege")
        .and(warp::delete())
        .and(warp::header("Authorization"))
        .and(warp::query::<RefundQuery>())
        .and(with_arc(repository.clone()))
        .and(with_arc(checker.clone()))
        .and_then(refund_handler);
    let health_route = warp::path!("manage" / "health")
        .and(warp::get())
        .and_then(health_check_handler);
    let routes = get_route
        .or(purchase_route)
        .or(refund_route)
        .or(health_route)
        .with(log);
    routes
}

pub async fn run_server(repository: Arc<Mutex<dyn PrivilegeRepository>>, port: u16, checker: Arc<Mutex<jwtchecker::JWTChecker>>) {
    let router = router(repository, checker);
    warp::serve(router)
        .run(([0, 0, 0, 0], port))
        .await
}
