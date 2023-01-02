use http::StatusCode;
use lazy_static::lazy_static;
use log;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{env, result::Result};
use urlencoding;

const GRAPHQL_URL: &str = "https://gql.twitch.tv/gql";

lazy_static! {
  static ref CLIENT_ID: String = env::var("TWITCH_CLIENT_ID").unwrap_or(String::from(""));

  // https://www.twitch.tv/speedgaming
  static ref CHANNEL_URL_PATTERNS: [Regex; 1] = [
    Regex::new(r"^https?://www\.twitch\.tv/(?P<channel_name>[^/?#]+)").unwrap(),
  ];

  // https://www.twitch.tv/videos/113837699
  // https://www.twitch.tv/gamesdonequick/video/113837699 (legacy url)
  // https://www.twitch.tv/gamesdonequick/v/113837699 (legacy url)
  // https://player.twitch.tv/?video=v113837699&parent=example.com ("v" is optional)
  static ref VIDEO_URL_PATTERNS: [Regex; 3] = [
    Regex::new(r"^https?://www\.twitch\.tv/videos/(?P<video_id>\d+)").unwrap(),
    Regex::new(r"^https?://www\.twitch\.tv/[^/]+/v(?:ideo)?/(?P<video_id>\d+)").unwrap(),
    Regex::new(r"^https?://player\.twitch\.tv/[^#]*[?&]video=v?(?P<video_id>\d+)").unwrap(),
  ];

  // https://clips.twitch.tv/AmazonianKnottyLapwingSwiftRage
  // https://www.twitch.tv/gamesdonequick/clip/ExuberantMiniatureSandpiperDogFace
  static ref CLIP_URL_PATTERNS: [Regex; 2] = [
    Regex::new(r"^https?://clips\.twitch\.tv/(?P<slug>[^/?#]+)").unwrap(),
    Regex::new(r"^https?://www\.twitch\.tv/[^/]+/clip/(?P<slug>[^/?#]+)").unwrap(),
  ];
}

// Channel
#[derive(Debug, Serialize, Deserialize)]
struct StreamResponseData {
  data: StreamPlaybackAccessTokenData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamPlaybackAccessTokenData {
  stream_playback_access_token: Option<PlaybackAccessToken>,
}

// Video
#[derive(Debug, Serialize, Deserialize)]
struct VideoResponseData {
  data: VideoPlaybackAccessTokenData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VideoPlaybackAccessTokenData {
  video_playback_access_token: Option<PlaybackAccessToken>,
}

// Clip
#[derive(Debug, Serialize, Deserialize)]
struct ClipResponseData {
  data: ClipData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClipData {
  clip: Clip,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlaybackAccessTokenData {
  playback_access_token: Option<PlaybackAccessToken>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Clip {
  playback_access_token: Option<PlaybackAccessToken>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlaybackAccessToken {
  signature: String,
  value: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClipTokenValue {
  clip_uri: String,
}

pub fn probe(url: &str) -> bool {
  // TODO: Return information about the match to avoid the need to do the matching again in the resolve function
  if CLIENT_ID.eq("") {
    return false;
  }

  for re in CLIP_URL_PATTERNS.iter() {
    if cfg!(debug_assertions) {
      log::info!("re: {:?}", re);
    }
    let ret = re.captures(url);
    if ret.is_none() {
      continue;
    }
    return true;
  }

  for re in VIDEO_URL_PATTERNS.iter() {
    if cfg!(debug_assertions) {
      log::info!("re: {:?}", re);
    }
    let ret = re.captures(url);
    if ret.is_none() {
      continue;
    }
    return true;
  }

  for re in CHANNEL_URL_PATTERNS.iter() {
    if cfg!(debug_assertions) {
      log::info!("re: {:?}", re);
    }
    let ret = re.captures(url);
    if ret.is_none() {
      continue;
    }
    return true;
  }

  return false;
}

pub async fn resolve(url: &str) -> Result<String, &'static str> {
  for re in CLIP_URL_PATTERNS.iter() {
    if cfg!(debug_assertions) {
      log::info!("re: {:?}", re);
    }
    let ret = re.captures(url);
    if ret.is_none() {
      continue;
    }
    if cfg!(debug_assertions) {
      log::info!("ret: {:?}", ret);
    }
    let slug = ret.unwrap().get(1).unwrap().as_str();
    if cfg!(debug_assertions) {
      log::info!("slug: {}", slug);
    }
    return clip_url(slug).await;
  }

  for re in VIDEO_URL_PATTERNS.iter() {
    if cfg!(debug_assertions) {
      log::info!("re: {:?}", re);
    }
    let ret = re.captures(url);
    if ret.is_none() {
      continue;
    }
    if cfg!(debug_assertions) {
      log::info!("ret: {:?}", ret);
    }
    let video_id = ret.unwrap().get(1).unwrap().as_str();
    if cfg!(debug_assertions) {
      log::info!("video_id: {}", video_id);
    }
    return video_url(video_id).await;
  }

  for re in CHANNEL_URL_PATTERNS.iter() {
    if cfg!(debug_assertions) {
      log::info!("re: {:?}", re);
    }
    let ret = re.captures(url);
    if ret.is_none() {
      continue;
    }
    if cfg!(debug_assertions) {
      log::info!("ret: {:?}", ret);
    }
    let channel_name = ret.unwrap().get(1).unwrap().as_str();
    if cfg!(debug_assertions) {
      log::info!("channel_name: {}", channel_name);
    }
    return channel_url(channel_name).await;
  }

  return Err("not found");
}

async fn channel_url(channel_name: &str) -> Result<String, &'static str> {
  // https://www.twitch.tv/directory/game/Perfect%20Dark
  // https://www.twitch.tv/recaps/annual
  if channel_name == "directory" || channel_name == "recaps" {
    return Err("unsupported channel name");
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
    .post(GRAPHQL_URL)
    .header("Client-ID", (*CLIENT_ID).as_str())
    .body(serde_json::to_string(&request_data).unwrap())
    .send()
    .await
    .expect("send graphql request");
  let response_status = response.status();
  let response_text = response.text().await.expect("read response data");

  if response_status != (StatusCode::OK) {
    log::error!("bad response: {} - {:?}", response_status, response_text);
    return Err("received non-200 response from Twitch");
  }

  let response_data: StreamResponseData = serde_json::from_str(response_text.as_str()).unwrap();

  if cfg!(debug_assertions) {
    log::info!("response_data: {:?}", response_data);
  }

  if response_data.data.stream_playback_access_token.is_none() {
    return Err("streamPlaybackAccessToken is null");
  }

  let access_token = response_data.data.stream_playback_access_token.unwrap();
  return Ok(format!(
    "https://usher.ttvnw.net/api/channel/hls/{}.m3u8?allow_source=true&allow_audio_only=true&sig={}&token={}",
    channel_name,
    urlencoding::encode(access_token.signature.as_str()),
    urlencoding::encode(access_token.value.as_str())
  ));
}

async fn video_url(video_id: &str) -> Result<String, &'static str> {
  let q = json!({
    "query": include_str!("videoPlaybackAccessToken.gql"),
    "variables": {
      "vodID": video_id,
      "platform": "web",
      "playerType": "site",
    },
  });
  let request_data = serde_json::to_string(&q).unwrap();

  let client = reqwest::Client::builder()
    .build()
    .expect("build reqwest client");
  let response = client
    .post(GRAPHQL_URL)
    .header("Client-ID", (*CLIENT_ID).as_str())
    .body(request_data)
    .send()
    .await
    .expect("send graphql request");
  let response_status = response.status();
  let response_text = response.text().await.expect("read response data");

  if response_status != (StatusCode::OK) {
    log::error!("bad response: {} - {:?}", response_status, response_text);
    return Err("received non-200 response from Twitch");
  }

  let response_data: VideoResponseData = serde_json::from_str(response_text.as_str()).unwrap();

  if cfg!(debug_assertions) {
    log::info!("response_data: {:?}", response_data);
  }

  if response_data.data.video_playback_access_token.is_none() {
    return Err("videoPlaybackAccessToken is null");
  }

  let access_token = response_data.data.video_playback_access_token.unwrap();

  return Ok(format!(
    "https://usher.ttvnw.net/vod/{}.m3u8?allow_source=true&allow_audio_only=true&sig={}&token={}",
    video_id,
    urlencoding::encode(access_token.signature.as_str()),
    urlencoding::encode(access_token.value.as_str())
  ));
}

async fn clip_url(slug: &str) -> Result<String, &'static str> {
  let q = json!({
    "query": include_str!("clip.gql"),
    "variables": {
      "slug": slug,
      "platform": "web",
      "playerType": "site",
    },
  });
  let request_data = serde_json::to_string(&q).unwrap();

  let client = reqwest::Client::builder()
    .build()
    .expect("build reqwest client");
  let response = client
    .post(GRAPHQL_URL)
    .header("Client-ID", (*CLIENT_ID).as_str())
    .body(request_data)
    .send()
    .await
    .expect("send graphql request");
  let response_status = response.status();
  let response_text = response.text().await.expect("read response data");

  if response_status != (StatusCode::OK) {
    log::error!("bad response: {} - {:?}", response_status, response_text);
    return Err("received non-200 response from Twitch");
  }

  let response_data: ClipResponseData = serde_json::from_str(response_text.as_str()).unwrap();

  if cfg!(debug_assertions) {
    log::info!("response_data: {:?}", response_data);
  }

  if response_data.data.clip.playback_access_token.is_none() {
    return Err("playbackAccessToken is null");
  }

  let access_token = response_data.data.clip.playback_access_token.unwrap();
  let clip_token_value: ClipTokenValue = serde_json::from_str(access_token.value.as_str()).unwrap();
  if cfg!(debug_assertions) {
    log::info!("clip_token_value: {:?}", clip_token_value);
  }

  return Ok(format!(
    "{}?sig={}&token={}",
    clip_token_value.clip_uri,
    urlencoding::encode(access_token.signature.as_str()),
    urlencoding::encode(access_token.value.as_str())
  ));
}
