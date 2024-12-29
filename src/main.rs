pub mod simulation;
pub mod timeline;
pub mod store;
pub mod handle;
pub mod multiverse;
pub mod multiverse_manager;

use std::{ops::Mul, sync::{mpsc::{self, Sender}, OnceLock}, thread};

use actix_web::{get, middleware, post, web, App, HttpResponse, HttpServer, Responder};
use handle::Handle;
use multiverse_manager::MultiverseCommand;
use simulation::{Body, Universe};

#[get("/")]
async fn hello() -> impl Responder {
    println!("hello hit");
    HttpResponse::Ok().body("Hello world!")
}

#[get("/echo/{val}")]
async fn echo(req_body: String, path: web::Path<(String,)>) -> HttpResponse {
    HttpResponse::Ok().json(format!("{{ val: {} }}", path.into_inner().0))
    //HttpResponse::Ok().body(format!("Got val: {}", path.into_inner().0))
}

#[get("/nodes")]
async fn fetch_nodes() -> impl Responder {
    let (tx, rx) = mpsc::channel();
    CHAN.get().unwrap().send(MultiverseCommand::GetNodes(tx));
    let nodes = rx.recv().expect("Failed to fetch nodes");
    let json = serde_json::to_string(&nodes);
    HttpResponse::Ok().json(json.unwrap())
}

#[get("/universe/{uuid}")]
async fn fetch_universe(path : web::Path<(String,)>) -> impl Responder {
    let (tx, rx) = mpsc::channel();
    let handle = Handle::new_from(&path.into_inner().0);
    CHAN.get().unwrap().send(MultiverseCommand::GetUniverse((handle, tx)));
    match rx.recv().expect("Failed to read universe") {
        Some(u) => HttpResponse::Ok().json(serde_json::to_string(&u).unwrap()),
        None => HttpResponse::Ok().body("Universe not found. Please double check the submitted UUID")
    }
}

#[get("/update_universe/{uuid}")]
async fn update_universe(path: web::Path<(String,)>) -> impl Responder {
    let (tx, rx) = mpsc::channel();
    
    HttpResponse::Ok()
}

async fn test_echo() -> impl Responder
{
    let (tx2, rx) = mpsc::channel();
    CHAN.get().unwrap().send(MultiverseCommand::Echo(tx2));
    println!("{}", rx.recv().unwrap());
    HttpResponse::Ok().body("got an echo!")
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

/*fn main() {
    let mut b1 = Body::new();
    b1.position.x = 100.0;
    b1.mass = 5.0;
    b1.velocity.z = 1.0;
    let mut b2 = Body::new();
    b2.mass = 1.0;
    let mut b3 = Body::new();
    b3.mass = 1.0;
    b3.position.y = 100.0;
    let mut u = Universe::new();
    u.add_body(b1);
    u.add_body(b2);
    u.add_body(b3);
    u.tick_for(1);
    for body in &u.bodies {
        println!("{},{},{}", &body.position.x, &body.position.y, &body.position.z);
    }
    let store = store::StoreSQL::new();
    let handle = store.save(u);
    println!("------------------");
    let u2: Universe = store.get(&handle).expect("Failed to fetch universe for some reason");
    for body in u2.bodies {
        println!("{},{},{}", &body.position.x, &body.position.y, &body.position.z);
    }
}*/

use rusqlite::{params, Connection, Result};
use store::Store;
use uuid::Uuid;

#[derive(Debug)]
struct Person {
    id: i32,
    name: String,
    data: Option<Vec<u8>>,
}

/*fn main() -> Result<()> {
    //let conn = Connection::open_in_memory()?;
    let conn = Connection::open("./people_db.sqlite")?;
    let conn2 = Connection::open("./people_db.sqlite")?;

    /*conn.execute(
        "CREATE TABLE person (
            id   INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            data BLOB
        )",
        (), // empty list of parameters.
    )?;*/
    let me = Person {
        id: 5,
        name: "Frank".to_string(),
        data: None,
    };
    conn.execute(
        "INSERT INTO person (name, data) VALUES (?1, ?2)",
        (&me.name, &me.data),
    )?;

    let mut stmt = conn.prepare("SELECT id, name, data FROM person")?;
    let person_iter = stmt.query_map([], |row| {
        Ok(Person {
            id: row.get(0)?,
            name: row.get(1)?,
            data: row.get(2)?,
        })
    })?;

    for person in person_iter {
        println!("Found person {:?}", person.unwrap());
    }
    Ok(())
}*/

static CHAN: OnceLock<Sender<MultiverseCommand>> = OnceLock::new();

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting multiverse...");
    let (tx, rx) = mpsc::channel();
    let multiverse_thread = thread::spawn(move || {
        multiverse_manager::start_multiverse(rx);
    });
    CHAN.set(tx);
    //let bound_echo = |tx: Sender<MultiverseCommand>| {test_echo(tx.clone())};
    println!("Starting webserver...");
    HttpServer::new(|| {
        let api_scope = web::scope("/api")
            .service(echo)
            .service(fetch_nodes)
            .service(fetch_universe)
            .route("/test", web::get().to(test_echo));
        App::new()
            .wrap(middleware::Logger::default())
            //.service(hello)
            .service(api_scope)
            //.route("/hey", web::get().to(manual_hello))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}