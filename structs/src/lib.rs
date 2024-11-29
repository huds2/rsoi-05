use serde::{Deserialize, Serialize};
use chrono::{NaiveDateTime, DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Privilege {
    pub id: i32,
    pub username: String,
    pub status: String,
    pub balance: i32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivilegeHistory {
    pub id: i32,
    pub privilege_id: i32,
    pub ticket_uid: Uuid,
    pub datetime: NaiveDateTime,
    pub balance_diff: i32,
    pub operation_type: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivilegeHistoryPost {
    pub username: String,
    pub ticket_uid: Uuid,
    pub balance_diff: i32,
    pub operation_type: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivilegeGet {
    pub balance: i32,
    pub status: String,
    pub history: Vec<PrivilegeHistoryGet>
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivilegeHistoryGet {
    pub date: String,
    pub ticketUid: Uuid,
    pub balanceDiff: i32,
    pub operationType: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub balance: i32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub tickets: Vec<TicketResponse>,
    pub privilege: Balance
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchasePost {
    pub ticket_uid: Uuid,
    pub price: i32,
    pub paid_from_balance: bool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseResponse {
    pub paid_by_money: i32,
    pub paid_by_bonuses: i32,
    pub balance: i32,
    pub status: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flight {
    pub id: i32,
    pub flight_number: String,
    pub datetime: DateTime<Utc>,
    pub from_airport_id: i32,
    pub to_airport_id: i32,
    pub price: i32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Airport {
    pub id: i32,
    pub name: String,
    pub city: String,
    pub country: Option<String>
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFlight {
    pub flightNumber: String,
    pub fromAirport: String,
    pub toAirport: String,
    pub date: String,
    pub price: i32
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFlightPage {
  pub page: usize,
  pub pageSize: usize,
  pub totalElements: usize,
  pub items: Vec<WebFlight>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub id: i32,
    pub ticket_uid: Uuid,
    pub username: String,
    pub flight_number: String,
    pub price: i32,
    pub status: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketPost {
    pub flight_number: String,
    pub price: i32,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketPostBalance {
    pub flightNumber: String,
    pub price: i32,
    pub paidFromBalance: bool,
}


#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketResponse {
    pub ticketUid: Uuid,
    pub flightNumber: String,
    pub fromAirport: String,
    pub toAirport: String,
    pub date: String,
    pub price: i32,
    pub status: String
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedPurchaseResponse {
    pub ticketUid: Uuid,
    pub flightNumber: String,
    pub fromAirport: String,
    pub toAirport: String,
    pub date: String,
    pub price: i32,
    pub paidByMoney: i32,
    pub paidByBonuses: i32,
    pub status: String,
    pub privilege: Balance
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub gateway: bool,
    pub flights: bool,
    pub tickets: bool,
    pub bonuses: bool
}
