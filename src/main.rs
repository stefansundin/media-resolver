use actix_web::{get, middleware, web, App, HttpResponse, HttpServer};
use env_logger;
use http::StatusCode;
use lazy_static::lazy_static;
use log;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{env, process};

const TWITCH_GRAPHQL_URL: &str = "https://gql.twitch.tv/gql";

lazy_static! {
  static ref TWITCH_CLIENT_ID: String = env::var("TWITCH_CLIENT_ID").unwrap_or(String::from(""));
}

#[derive(Debug, Serialize, Deserialize)]
struct StreamResponseData {
  data: StreamPlaybackAccessTokenData,
}

#[derive(Debug, Serialize, Deserialize)]
struct VideoResponseData {
  data: VideoPlaybackAccessTokenData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamPlaybackAccessTokenData {
  stream_playback_access_token: Option<PlaybackAccessToken>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VideoPlaybackAccessTokenData {
  video_playback_access_token: Option<PlaybackAccessToken>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlaybackAccessToken {
  signature: String,
  value: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

  if TWITCH_CLIENT_ID.eq("") {
    log::error!("TWITCH_CLIENT_ID is not defined!");
    process::exit(1);
  }

  HttpServer::new(|| {
    App::new()
      .service(channel_access_token)
      .service(vod_access_token)
      .wrap(middleware::Logger::new(
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

#[get("/api/channels/{channel_name}/access_token")]
async fn channel_access_token(path: web::Path<String>) -> HttpResponse {
  let channel_name = path.into_inner();
  if cfg!(debug_assertions) {
    log::info!("channel_name: {}", channel_name);
  }

  let request_data = json!({
    "query": include_str!("streamPlaybackAccessToken.gql"),
    "variables": {
      "channelName": channel_name,
      "platform": "web",
      "playerType": "site",
    },
  });

  let client = reqwest::Client::builder()
    .build()
    .expect("build reqwest client");
  let response = client
    .post(TWITCH_GRAPHQL_URL)
    .header("Client-ID", (*TWITCH_CLIENT_ID).as_str())
    .body(serde_json::to_string(&request_data).unwrap())
    .send()
    .await
    .expect("send graphql request");
  let response_status = response.status();
  let response_text = response.text().await.expect("read response data");

  if response_status != (StatusCode::OK) {
    log::error!("bad response: {} - {:?}", response_status, response_text);
    return HttpResponse::InternalServerError().json(json!({
      "error": "received non-200 response from Twitch",
    }));
  }

  let response_data: StreamResponseData = serde_json::from_str(response_text.as_str()).unwrap();

  if cfg!(debug_assertions) {
    log::info!("response_data: {:?}", response_data);
  }

  if response_data.data.stream_playback_access_token.is_none() {
    return HttpResponse::InternalServerError().json(json!({
      "error": "streamPlaybackAccessToken is null",
    }));
  }

  let access_token = response_data.data.stream_playback_access_token.unwrap();
  return HttpResponse::Ok().json(json!({
    "sig": access_token.signature,
    "token": access_token.value,
  }));
}

#[get("/api/vods/{vod_id}/access_token")]
async fn vod_access_token(path: web::Path<String>) -> HttpResponse {
  let vod_id = path.into_inner();
  if cfg!(debug_assertions) {
    log::info!("vod_id: {}", vod_id);
  }

  if !vod_id.chars().all(|c| c.is_ascii_digit()) {
    return HttpResponse::InternalServerError().json(json!({
      "error": "vod_id is not numeric",
    }));
  }

  let q = json!({
    "query": include_str!("videoPlaybackAccessToken.gql"),
    "variables": {
      "vodID": vod_id,
      "platform": "web",
      "playerType": "site",
    },
  });
  let request_data = serde_json::to_string(&q).unwrap();

  let client = reqwest::Client::builder()
    .build()
    .expect("build reqwest client");
  let response = client
    .post(TWITCH_GRAPHQL_URL)
    .header("Client-ID", (*TWITCH_CLIENT_ID).as_str())
    .body(request_data)
    .send()
    .await
    .expect("send graphql request");
  let response_status = response.status();
  let response_text = response.text().await.expect("read response data");

  if response_status != (StatusCode::OK) {
    log::error!("bad response: {} - {:?}", response_status, response_text);
    return HttpResponse::InternalServerError().json(json!({
      "error": "received non-200 response from Twitch",
    }));
  }

  let response_data: VideoResponseData = serde_json::from_str(response_text.as_str()).unwrap();

  if cfg!(debug_assertions) {
    log::info!("response_data: {:?}", response_data);
  }

  if response_data.data.video_playback_access_token.is_none() {
    return HttpResponse::InternalServerError().json(json!({
      "error": "videoPlaybackAccessToken is null",
    }));
  }

  let access_token = response_data.data.video_playback_access_token.unwrap();
  return HttpResponse::Ok().json(json!({
    "sig": access_token.signature,
    "token": access_token.value,
  }));
}
