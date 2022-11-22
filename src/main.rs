use actix_web::web::{BufMut, BytesMut};
use actix_web::{
    error, middleware,
    web::{self},
    App, Error, HttpRequest, HttpResponse, HttpServer,
};
use awc::Client;
use clap::Parser;
use hyper::StatusCode;
use serde::Deserialize;
use std::sync::{Arc, RwLock};

mod utils;
mod cache;
mod tls;

use crate::cache::Cache;
use crate::utils::bytes_to_stream;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct HashedRequest {
    pub uri: String,
    pub method: String,
}

#[derive(Deserialize)]
struct QueryParams {
    url: String,
}

async fn forward(
    req: HttpRequest,
    payload: web::Payload,
    params: web::Query<QueryParams>,
    client: web::Data<Client>,
    cache: web::Data<Arc<RwLock<Cache<HashedRequest, BytesMut>>>>,
) -> Result<HttpResponse, Error> {
    let hashed_request = HashedRequest {
        uri: req.uri().to_string(),
        method: req.method().to_string(),
    };

    let mut cache = cache.write().unwrap();

    // Check to see if we previously cached this request.
    let result = cache.get(&hashed_request);

    // Return the cached response if available
    if let Some(cached_response) = result {
        let mut client_resp = HttpResponse::build(StatusCode::OK);

        let stream = bytes_to_stream(cached_response.clone());

        log::info!("serving cached response for request {}", req.uri());

        return Ok(client_resp.streaming(stream));
    }

    let forwarded_req = client
        .request_from(params.url.as_str(), req.head())
        .no_decompress();

    let forwarded_req = match req.head().peer_addr {
        Some(addr) => forwarded_req.insert_header(("x-forwarded-for", format!("{}", addr.ip()))),
        None => forwarded_req,
    };

    let mut res = forwarded_req
        .send_stream(payload)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let mut client_resp = HttpResponse::build(res.status());

    // Remove `Connection` as per
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
    for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
        client_resp.insert_header((header_name.clone(), header_value.clone()));
    }

    // Convert response body into bytes for caching
    let mut bytes_mut = actix_web::web::BytesMut::new();
    bytes_mut.put(res.body().await?);

    cache.set(hashed_request, bytes_mut.clone());

    let stream = bytes_to_stream(bytes_mut);
    Ok(client_resp.streaming(stream))
}

#[derive(clap::Parser, Debug)]
struct CliArguments {
    #[arg(default_value_t = String::from("127.0.0.1"))]
    listen_addr: String,

    #[arg(default_value_t = 8080)]
    listen_port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let args = CliArguments::parse();

    log::info!(
        "starting HTTP server at http://{}:{}",
        &args.listen_addr,
        args.listen_port
    );

    // Load TLS configuration
    let config = tls::load_rustls_config();

    let cache = Arc::new(RwLock::new(Cache::<HashedRequest, BytesMut>::new()));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Client::default()))
            .app_data(web::Data::new(cache.clone()))
            .wrap(middleware::Logger::default())
            .default_service(web::to(forward))
    })
    .bind_rustls("127.0.0.1:8080", config)?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use actix_web::{http, test, web, App};

    use super::*;

    #[actix_web::test]
    async fn test_url_query_param_status_ok() {
        let app = test::init_service(App::new().default_service(web::to(forward))).await;
        let req = test::TestRequest::default()
            .uri("/?url=https://blockstream.info/api/blocks/0")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_no_url_param_errors() {
        let app = test::init_service(App::new().default_service(web::to(forward))).await;
        let req = test::TestRequest::default().to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }
}
