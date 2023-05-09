use std::{collections::HashMap, fs::File, io::Result, sync::Mutex};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
#[derive(Serialize)]
struct GetPost {
    result: String,
    status: String,
}
#[derive(Deserialize)]
struct SetPostInput {
    id: String,
    value: String,
    save: Option<bool>,
}
#[derive(Serialize)]
struct SetPostOutput {
    id: String,
    value: String,
    status: String,
}
struct AppState {
    hash_map: Mutex<HashMap<String, String>>,
}
#[actix_web::main]
async fn main() -> Result<()> {
    let data: HashMap<String, String> = match File::open("./db.json") {
        Ok(e) => serde_json::from_reader(e).unwrap(),
        Err(e) => {
            println!("no db.json detected \n {}", e);
            HashMap::new()
        }
    };
    let state = web::Data::new(AppState {
        hash_map: Mutex::new(data.clone()),
    });
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(get_get)
            .service(set_post)
            .service(get_post)
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}
fn save(data: &HashMap<String, String>) {
    let file = File::create("./db.json").unwrap();
    serde_json::to_writer(file, data).unwrap();
}
#[get("/")]
async fn get_get(
    state: web::Data<AppState>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let id = query.get("id");
    return match id {
        Some(dt) => match state.hash_map.lock().expect("something went wrong").get(dt) {
            Some(res) => HttpResponse::Ok().body(res.to_string()),
            None => HttpResponse::NotFound().body("Not found"),
        },
        None => HttpResponse::BadRequest().body("Empty id"),
    };
}
#[post("/get")]
async fn get_post(
    req: web::Json<HashMap<String, String>>,
    state: web::Data<AppState>,
) -> impl Responder {
    return match req.get("id") {
        Some(id) => match state.hash_map.lock().expect("something went wrong").get(id) {
            Some(dt) => HttpResponse::Ok().json(GetPost {
                result: dt.to_string(),
                status: "ok".to_string(),
            }),
            None => HttpResponse::NotFound().json(GetPost {
                result: "".to_string(),
                status: "not_found".to_string(),
            }),
        },
        None => HttpResponse::BadRequest().body("Empty id"),
    };
}
#[post("/")]
async fn set_post(state: web::Data<AppState>, req: web::Json<SetPostInput>) -> impl Responder {
    let ret = match (&req.id, &req.value) {
        (e, f) => {
            let mut map = state.hash_map.lock().expect("something went wrong");
            map.insert(e.to_string(), f.to_string());

            HttpResponse::Ok().json(SetPostOutput {
                id: e.to_string(),
                value: f.to_string(),
                status: "ok".to_string(),
            })
        }
        _ => HttpResponse::BadRequest().json(SetPostOutput {
            id: "".to_string(),
            value: "".to_string(),
            status: "fail".to_string(),
        }),
    };
    match req.save {
        Some(true) => save(&state.hash_map.lock().unwrap()),
        _ => println!("not saved"),
    }
    return ret;
}
