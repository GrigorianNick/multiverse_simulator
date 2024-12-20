pub mod simulation;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use simulation::{Body, Universe};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

fn main() {
    let mut b1 = Body::new();
    b1.position.x = 100.0;
    b1.mass = 5.0;
    b1.velocity.z = 1.0;
    let mut b2 = Body::new();
    b2.mass = 1.0;
    let mut u = Universe::new();
    u.add_body(b1);
    u.add_body(b2);
    u.tick_for(1000);
    for body in u.bodies {
        println!("{},{},{}", &body.position.x, &body.position.y, &body.position.z);
    }
}

/*#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut b1 = Body::new();
    b1.position.x = 10.0;
    b1.mass = 1.0;
    let mut b2 = Body::new();
    b2.mass = 1.0;
    let mut u = Universe::new();
    u.add_body(b1);
    u.add_body(b2);
    u.tick();
    for body in u.bodies {
        println!("{},{},{}", &body.position.x, &body.position.y, &body.position.z);
    }
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}*/