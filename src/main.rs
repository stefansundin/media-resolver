use std::{env, net, process};

use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::json;

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
  stream_playback_access_token: PlaybackAccessToken,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VideoPlaybackAccessTokenData {
  video_playback_access_token: PlaybackAccessToken,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlaybackAccessToken {
  signature: String,
  value: String,
}

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();

  if TWITCH_CLIENT_ID.eq("") {
    tracing::error!("TWITCH_CLIENT_ID is not defined!");
    process::exit(1);
  }

  let app = Router::new()
    .route(
      "/api/channels/:channel_name/access_token",
      get(channel_access_token),
    )
    .route("/api/vods/:vod_id/access_token", get(vod_access_token));

  let addr = net::SocketAddr::from(([0, 0, 0, 0], 8080));
  tracing::info!("listening on {}", addr);
  axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .await
    .unwrap();
}

async fn channel_access_token(Path(channel_name): Path<String>) -> impl IntoResponse {
  if cfg!(debug_assertions) {
    tracing::info!("channel_name: {}", channel_name);
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
    tracing::error!("bad response: {} - {:?}", response_status, response_text);
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(json!({
        "error": "received non-200 response from Twitch",
      })),
    );
  }

  let response_data: StreamResponseData = serde_json::from_str(response_text.as_str()).unwrap();

  if cfg!(debug_assertions) {
    tracing::info!("response_data: {:?}", response_data);
  }

  return (
    StatusCode::OK,
    Json(json!({
      "sig": response_data.data.stream_playback_access_token.signature,
      "token": response_data.data.stream_playback_access_token.value,
    })),
  );
}

async fn vod_access_token(Path(vod_id): Path<String>) -> impl IntoResponse {
  if cfg!(debug_assertions) {
    tracing::info!("vod_id: {}", vod_id);
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
    tracing::error!("bad response: {} - {:?}", response_status, response_text);
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(json!({
        "error": "received non-200 response from Twitch",
      })),
    );
  }

  let response_data: VideoResponseData = serde_json::from_str(response_text.as_str()).unwrap();

  if cfg!(debug_assertions) {
    tracing::info!("response_data: {:?}", response_data);
  }

  return (
    StatusCode::OK,
    Json(json!({
      "sig": response_data.data.video_playback_access_token.signature,
      "token": response_data.data.video_playback_access_token.value,
    })),
  );
}
