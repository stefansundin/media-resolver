pub mod twitch;

use actix_web::{get, middleware, web, App, HttpResponse, HttpServer};
use env_logger;
use log;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;

#[derive(Debug, Deserialize)]
pub struct ResolveRequest {
  url: String,
  output: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PlaylistItem {
  path: String,
  name: String,
  description: Option<String>,
  language: Option<String>,
  artist: Option<String>,
  genre: Option<String>,
  date: Option<String>,
  duration: Option<usize>, // seconds
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
  let output_string = q.output.unwrap_or_default();
  let output = output_string.as_str();

  if twitch::probe(url) {
    let playlist = match twitch::resolve(url).await {
      Ok(v) => v,
      Err(e) => {
        log::error!("error: {}", e);
        let mut error_status = if output == "json" {
          // VLC playlist parsers can't read the data of non-200 responses
          HttpResponse::Ok()
        } else {
          HttpResponse::InternalServerError()
        };
        return error_status.json(json!({
          "error": e,
        }));
      }
    };
    if cfg!(debug_assertions) {
      log::info!("playlist: {:?}", playlist);
    }

    if output == "json" {
      return HttpResponse::Ok().json(playlist);
    } else if playlist.len() > 0 {
      return HttpResponse::TemporaryRedirect()
        .append_header(("Location", playlist.first().unwrap().path.as_str()))
        .finish();
    } else {
      return HttpResponse::NotFound().finish();
    }
  }
  return HttpResponse::NotFound().finish();
}
