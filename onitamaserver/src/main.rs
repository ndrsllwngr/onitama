#[macro_use]
extern crate log;
extern crate pretty_env_logger;

use std::convert::TryFrom;
use std::path;

use actix::prelude::*;
use actix_files::Files;
use actix_web::{App, HttpServer, web};
use actix_web::dev::Service;
use actix_web::http::header::{CACHE_CONTROL, CacheControl, CacheDirective};
use actix_web::http::HeaderValue;

use crate::rooms::OnitamaServer;
use crate::routes::{ai_room, create_room, join_room, ServerData};

mod rooms;
mod messages;
mod routes;
mod agents;
mod utils;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();
    let server_addr = OnitamaServer::new().start();
    let data = ServerData { server_addr };
    let data = web::Data::new(data);
    let mut built_path = path::Path::new("./build");
    if !built_path.exists() {
        built_path = path::Path::new("../build");
    }
    info!("Does build path exist ({}): {}", built_path.as_os_str().to_string_lossy(), built_path.exists());
    info!("Starting server");
    HttpServer::new(move || {
        let app = App::new()
            // Cache all requests to paths in /static otherwise don't cache
            .wrap_fn(|req, srv| {
                let is_static = req.path().starts_with("/static") || req.path().ends_with(".wasm");
                let cache_static = match is_static {
                    true => CacheControl(vec![CacheDirective::MaxAge(86400)]).to_string(),
                    false => CacheControl(vec![CacheDirective::NoCache]).to_string(),
                };
                let fut = srv.call(req);
                async {
                    let mut res = fut.await?;
                    let cache_control: HeaderValue = HeaderValue::try_from(cache_static).expect("Oops");
                    res.headers_mut().insert(
                        CACHE_CONTROL, cache_control,
                    );
                    Ok(res)
                }
            })
            .app_data(data.clone())
            .service(
                web::scope("/ws")
                    .route("/ai", web::get().to(ai_room))
                    .route("/{key}", web::get().to(join_room))
                    .route("/", web::get().to(create_room))
            );
        match built_path.exists() {
            true => app
                .service(
                    Files::new("/", built_path)
                        .index_file("index.html")
                ),
            false => app,
        }
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
