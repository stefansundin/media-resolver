pub mod twitch;

use actix_web::{get, middleware, web, App, HttpResponse, HttpServer};
use env_logger;
use log;
use serde::Deserialize;
use serde_json::json;
use std::env;

#[derive(Debug, Deserialize)]
pub struct ResolveRequest {
  url: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

  HttpServer::new(|| {
    App::new().service(resolve).wrap(middleware::Logger::new(
      env::var("ACCESS_LOG_FORMAT")
        .unwrap_or(String::from(
          r#"%{r}a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#,
        ))
        .as_str(),
    ))
  })
  .bind(("0.0.0.0", 8080))?
  .run()
  .await
}

#[get("/resolve")]
async fn resolve(web::Query(q): web::Query<ResolveRequest>) -> HttpResponse {
  if cfg!(debug_assertions) {
    log::info!("url: {}", q.url);
  }
  let url = q.url.as_str();

  if twitch::probe(url) {
    let media_url = match twitch::resolve(url).await {
      Ok(v) => v,
      Err(e) => {
        log::error!("error: {}", e);
        return HttpResponse::InternalServerError().json(json!({
          "error": e,
        }));
      }
    };
    if cfg!(debug_assertions) {
      log::info!("media_url: {:?}", media_url);
    }

    return HttpResponse::TemporaryRedirect()
      .append_header(("Location", media_url))
      .body("");
  }
  return HttpResponse::NotFound().body("");
}
