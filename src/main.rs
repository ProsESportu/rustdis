use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use crypto::scrypt::{scrypt_check, scrypt_simple, ScryptParams};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{Result, Write},
    sync::Mutex,
};
#[derive(Serialize)]
struct GetPost {
    result: String,
    status: String,
}
#[derive(Deserialize, Debug)]
struct GetInput {
    id: String,
    password: String,
    user: String,
    value: Option<String>,
    save: Option<bool>,
}

#[derive(Deserialize)]
struct SetInput {
    id: String,
    value: String,
    save: Option<bool>,
    user: String,
    password: String,
}
#[derive(Serialize)]
struct SetOutput {
    id: String,
    value: String,
    status: String,
}
struct AppState {
    hash_map: Mutex<HashMap<String, String>>,
    users: HashMap<String, String>,
}
#[actix_web::main]
async fn main() -> Result<()> {
    let users: HashMap<String, String> = match File::open("./users.json") {
        Ok(e) => serde_json::from_reader(e).unwrap(),
        Err(e) => {
            println!("no users.json detected \n {}", e);
            let mut pass = String::new();
            print!("enter the root password:");
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut pass).unwrap();
            let hash = scrypt_simple(&pass, &ScryptParams::new(15, 8, 1)).unwrap();
            let mut json = HashMap::new();
            json.insert("root".to_string(), hash);
            let file = File::create("./users.json").unwrap();
            serde_json::to_writer(file, &json).unwrap();
            json
        }
    };
    let data: HashMap<String, String> = match File::open("./db.json") {
        Ok(e) => serde_json::from_reader(e).unwrap(),
        Err(e) => {
            println!("no db.json detected \n {}", e);
            HashMap::new()
        }
    };
    let state = web::Data::new(AppState {
        hash_map: Mutex::new(data.clone()),
        users: users.clone(),
    });
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
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
#[post("/get")]
async fn get_post(req: web::Json<GetInput>, state: web::Data<AppState>) -> impl Responder {
    match state.users.get(&req.user) {
        Some(e) => match scrypt_check(&req.password, e) {
            Ok(_) => {
                match state
                    .hash_map
                    .lock()
                    .expect("something went wrong")
                    .get(&req.id)
                {
                    Some(dt) => HttpResponse::Ok().json(GetPost {
                        result: dt.to_string(),
                        status: "ok".to_string(),
                    }),
                    None => HttpResponse::NotFound().json(GetPost {
                        result: "".to_string(),
                        status: "not_found".to_string(),
                    }),
                }
            }
            Err(_) => HttpResponse::Forbidden().body("wrong password"),
        },
        None => HttpResponse::Forbidden().body("no such user"),
    }
}

#[post("/set")]
async fn set_post(state: web::Data<AppState>, req: web::Json<SetInput>) -> impl Responder {
    match state.users.get(&req.user) {
        Some(e) => match scrypt_check(&req.password, &e) {
            Ok(_) => {
                let mut map = state.hash_map.lock().expect("something went wrong");
                map.insert(req.id.to_string(), req.value.to_string());

                match req.save {
                    Some(true) => save(&state.hash_map.lock().unwrap()),
                    _ => println!("not saved"),
                }

                HttpResponse::Ok().json(SetOutput {
                    id: req.id.to_string(),
                    value: req.value.to_string(),
                    status: "ok".to_string(),
                })
            }
            Err(_)=>HttpResponse::Forbidden().body("wrong password")
        },
        None => HttpResponse::Forbidden().body("no such user"),
    }
}
