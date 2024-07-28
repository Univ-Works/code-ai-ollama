mod services {
    pub mod ollama;
    pub mod websocket;
}

use actix_web::{Error, middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use env_logger::Env;
use services::{ollama::{chatting, create_dynamic_exercise}, websocket::MyWs};

use serde::Deserialize;

use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize)]
struct DynamicExerciseRequest {
    language: String,
    topic: String,
    sub_topic: String
}

#[derive(Deserialize)]
struct ChattingRequest {
    message: String
}

/*async fn dynamic_exercise(body: web::Json<DynamicExerciseRequest>) -> Result<HttpResponse, Error> {
    create_dynamic_exercise(
        body.language.clone(),
        body.topic.clone(),
        body.sub_topic.clone()
    ).await?;

    Ok(HttpResponse::Ok().finish())
    /*let resp = ws::start(websocket::MyWs::new(), &request, stream);
    println!("{:?}", resp);

    resp*/
}*/

async fn chat_with_him(req: HttpRequest, stream: web::Payload/*body: web::Json<ChattingRequest>*/) -> Result<HttpResponse, Error> {
    ws::start(MyWs::new(), &req, stream)
    /*chatting(body.message.clone()).await?;
    Ok(HttpResponse::Ok().finish())*/
    /*chatting(message).await?;
    Ok(HttpResponse::Ok().finish())*/
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default()
        .default_filter_or("info")).init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .route("/chatting", web::get().to(chat_with_him))
    })    
        .workers(8)
        .bind(("127.0.0.1", 7072))?
        .run().await
}
//.route("/chatting", web::post().to(chat_with_him)))