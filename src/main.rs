use actix_files as fs;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use std::time::Instant;

/// WebSocket entry point
async fn game_ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, actix_web::Error> {
    ws::start(GameSession { hb: Instant::now() }, &req, stream)
}

struct GameSession {
    hb: Instant,
}

impl actix::Actor for GameSession {
    type Context = ws::WebsocketContext<Self>;
}

impl actix::StreamHandler<Result<ws::Message, ws::ProtocolError>> for GameSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Text(text)) => {
                // Echo the movement data back to all clients
                // In a production app, you would use an Addr to broadcast to other actors
                ctx.text(text);
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

async fn index() -> impl Responder {
    HttpResponse::Ok().content_type("text/html").body(include_str!("../static/index.html"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server starting at http://0.0.0.0:8080");

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/ws", web::get().to(game_ws))
            .service(fs::Files::new("/static", "./static"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
