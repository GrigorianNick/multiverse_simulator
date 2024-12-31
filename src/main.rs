pub mod simulation;
pub mod timeline;
pub mod store;
pub mod handle;
pub mod multiverse;
pub mod multiverse_manager;

use std::{sync::{mpsc::{self, Sender}, OnceLock}, thread};

use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder};
use handle::Handle;
use multiverse::BranchParams;
use multiverse_manager::MultiverseCommand;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use simulation::Pos;

#[get("/")]
async fn hello() -> impl Responder {
    println!("hello hit");
    HttpResponse::Ok().body("Hello world!")
}

#[get("/advance/{uuid}/{amount}")]
async fn advance_node(path: web::Path<(String, i32,)>) -> impl Responder {
    let vals = path.into_inner();
    let handle = Handle::new_from(&vals.0);
    CHAN.get().unwrap().send(MultiverseCommand::AdvanceNode((handle, vals.1)));
    HttpResponse::Ok().body("Node advanced")
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct BranchArgs{
    deltas: Vec<BranchParams>,
    duration: i32
}

#[get("/branch/{uuid}")]
async fn branch_node(path: web::Path<(String,)>/*, json: web::Json<BranchArgs>*/) -> impl Responder{
    let mut params = vec![];
    let mut p1 = BranchParams::default();
    p1.mass = Some(0.3);
    p1.position = Some(Pos{x: 0.0, y: 1.0, z: -0.5});
    params.push(p1);
    let mut p2 = BranchParams::default();
    p2.mass = Some(0.6);
    p2.position = Some(Pos{x: -1.0, y: 0.0, z: 0.0});
    p2.velocity = Some(Pos{x: 0.1, y: 0.1, z: 0.1});
    params.push(p2);
    let target_handle = Handle::new_from(&path.into_inner().0.to_string());
    CHAN.get().unwrap().send(MultiverseCommand::Branch((target_handle, params, 10)));
    HttpResponse::Ok()
}

#[get("/nodes")]
async fn fetch_nodes() -> impl Responder {
    let (tx, rx) = mpsc::channel();
    CHAN.get().unwrap().send(MultiverseCommand::GetNodes(tx));
    let nodes = rx.recv().expect("Failed to fetch nodes");
    let json = serde_json::to_string_pretty(&nodes);
    HttpResponse::Ok().json(json.unwrap())
}

#[get("/node/{uuid}")]
async fn fetch_node(path: web::Path<(String,)>) -> impl Responder {
    let (tx, rx) = mpsc::channel();
    CHAN.get().unwrap().send(MultiverseCommand::GetNode((Handle::new_from(&path.into_inner().0), tx)));
    match rx.recv() {
        Ok(opt) => HttpResponse::Ok().json(serde_json::to_string_pretty(&opt.unwrap()).unwrap()),
        Err(_) => HttpResponse::NotFound().body("Failed to find uuid"),
    }
}

#[get("/schema")]
async fn schema() -> impl Responder {
    let schema = schema_for!(BranchParams);
    HttpResponse::Ok().body(serde_json::to_string_pretty(&schema).unwrap())
}

#[get("/universe/{uuid}")]
async fn fetch_universe(path : web::Path<(String,)>) -> impl Responder {
    let (tx, rx) = mpsc::channel();
    let handle = Handle::new_from(&path.into_inner().0);
    CHAN.get().unwrap().send(MultiverseCommand::GetUniverse((handle, tx)));
    match rx.recv().expect("Failed to read universe") {
        Some(u) => HttpResponse::Ok().json(serde_json::to_string_pretty(&u).unwrap()),
        None => HttpResponse::Ok().body("Universe not found. Please double check the submitted UUID")
    }
}

#[get("/timeline/{uuid}")]
async fn fetch_timeline(path: web::Path<(String,)>) -> impl Responder {
    let (tx, rx) = mpsc::channel();
    let handle = Handle::new_from(&path.into_inner().0);
    CHAN.get().unwrap().send(MultiverseCommand::GetTimneline((handle, tx)));
    let timeline = rx.recv().unwrap();
    HttpResponse::Ok().json(serde_json::to_string_pretty(&timeline).unwrap())
}


static CHAN: OnceLock<Sender<MultiverseCommand>> = OnceLock::new();

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting multiverse...");
    let (tx, rx) = mpsc::channel();
    let multiverse_thread = thread::spawn(move || {
        multiverse_manager::start_multiverse(rx);
    });
    CHAN.set(tx);
    println!("Starting webserver...");
    HttpServer::new(|| {
        let api_scope = web::scope("/api")
            .service(fetch_nodes)
            .service(fetch_universe)
            .service(fetch_timeline)
            .service(fetch_node)
            .service(branch_node)
            .service(advance_node);
        App::new()
            .wrap(middleware::Logger::default())
            .service(api_scope)
            .service(schema)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}