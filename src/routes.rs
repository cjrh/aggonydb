use crate::db::MyPool;
use crate::schema::{add_dataset_if_missing, add_event, count};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use log;
use serde::Deserialize;

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
pub async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

pub async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Value {
    Int(i64),
    Bool(bool),
    Float(f64),
    Str(String),
}

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Deserialize, Debug)]
struct AddPayload {
    dataset: String,
    fields: HashMap<String, Value>,
}

#[post("/add")]
pub async fn add(
    pool: web::Data<Arc<Mutex<MyPool>>>,
    body: web::Json<AddPayload>,
) -> impl Responder {
    let pool = pool.lock().unwrap();
    let dataset_id = add_dataset_if_missing(&pool, &body.dataset).await.unwrap();
    log::debug!("Dataset id: {}", dataset_id);

    log::debug!("{:?}", body.fields);
    for (k, v) in body.fields.iter() {
        log::debug!("formatted v: {}", format!("{:?}", v));
        let vs = match v {
            Value::Int(x) => format!("{}", x),
            Value::Bool(x) => format!("{}", x),
            Value::Float(x) => format!("{}", x),
            Value::Str(x) => format!("{}", x),
        };
        add_event(&pool, dataset_id, k, &vs).await.unwrap();
    }

    HttpResponse::Ok().body("ok")
}

#[get("counts/{dataset}/{field}/{value}")]
pub async fn get_value_count(
    pool: web::Data<Arc<Mutex<MyPool>>>,
    path: web::Path<(String, String, String)>,
) -> impl Responder {
    let path = path.into_inner();
    let (dataset, field_name, value) = path;
    let pool = pool.lock().unwrap();
    let result = count(&pool, &dataset, &field_name, &value).await.unwrap();
    HttpResponse::Ok().body(format!("{}", result))
}

#[get("counts/{dataset}/{field}")]
pub async fn get_field_count(
    pool: web::Data<Arc<Mutex<MyPool>>>,
    path: web::Path<(String, String, String)>,
) -> impl Responder {
    let path = path.into_inner();
    let (dataset, field_name, value) = path;
    let pool = pool.lock().unwrap();
    let result = count(&pool, &dataset, &field_name, &value).await.unwrap();
    HttpResponse::Ok().body(format!("{}", result))
}
