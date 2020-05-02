//! Example of login and logout using redis-based sessions
//!
//! Every request gets a session, corresponding to a cache entry and cookie.
//! At login, the session key changes and session state in cache re-assigns.
//! At logout, session state in cache is removed and cookie is invalidated.
//!
use actix_web::{
    middleware, App, HttpServer,
    web::{get, post, resource},
};
use listenfd::ListenFd;
use rand::Rng;

mod handler;
use handler::{redis_session, favicon, index, count_up, login, logout, asset};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info,actix_redis=info");
    env_logger::init();

    let mut listenfd = ListenFd::from_env();
    let private_key = rand::thread_rng().gen::<[u8; 32]>();

    let mut server = HttpServer::new(move || {
        App::new()
            // redis session middleware
            .wrap(redis_session(&private_key, "sess", "actix-redis").unwrap())
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
            .route("/favicon.ico", get().to(favicon))
            .service(resource("/user").route(get().to(index)))
            .service(resource("/count_up").route(post().to(count_up)))
            .service(resource("/login").route(post().to(login)))
            .service(resource("/logout").route(post().to(logout)))
            .service(asset().unwrap())
    });
    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)?
    }else{
        server.bind("127.0.0.1:8080")?
    };
    server.run().await
}

