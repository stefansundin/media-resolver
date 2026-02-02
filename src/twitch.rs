use log;
use regex::Regex;
use reqwest::StatusCode;
use serde_json::json;
use std::{borrow::Cow, result::Result, sync::OnceLock};
use urlencoding;

use crate::PlaylistItem;
use crate::utils;

pub mod types;

const GRAPHQL_URL: &str = "https://gql.twitch.tv/gql";

fn channel_url_patterns() -> &'static [Regex] {
  static CHANNEL_URL_PATTERNS: OnceLock<[Regex; 1]> = OnceLock::new();
  CHANNEL_URL_PATTERNS.get_or_init(|| {
    [
      // https://www.twitch.tv/speedgaming
      Regex::new(r"^https?://www\.twitch\.tv/(?P<channel_name>[^/?#]+)").unwrap(),
    ]
  })
}

fn channel_videos_url_patterns() -> &'static [Regex] {
  static CHANNEL_VIDEOS_URL_PATTERNS: OnceLock<[Regex; 1]> = OnceLock::new();
  CHANNEL_VIDEOS_URL_PATTERNS.get_or_init(|| {
    [
      // https://www.twitch.tv/speedgaming/videos
      // https://www.twitch.tv/speedgaming/videos?filter=all&sort=time
      // https://www.twitch.tv/speedgaming/videos?filter=archives&sort=time
      // https://www.twitch.tv/speedgaming/videos?filter=archives&sort=views
      // https://www.twitch.tv/speedgaming/videos?filter=highlights&sort=time
      // https://www.twitch.tv/speedgaming/videos?filter=all&sort=time&cursor=1705053235|21|2023-01-12T11:49:13Z
      // TODO: Should probably parse the query string in another way
      Regex::new(r"^https?://www\.twitch\.tv/(?P<channel_name>[^/?#]+)/videos(?:[?&#](?:filter=(?P<filter>[^&#]+)|sort=(?P<sort>[^&#]+)|cursor=(?P<cursor>[^&#]+)))*").unwrap(),
    ]
  })
}

fn video_url_patterns() -> &'static [Regex] {
  static VIDEO_URL_PATTERNS: OnceLock<[Regex; 3]> = OnceLock::new();
  VIDEO_URL_PATTERNS.get_or_init(|| {
    [
      // https://www.twitch.tv/videos/113837699
      Regex::new(r"^https?://www\.twitch\.tv/videos/(?P<video_id>\d+)").unwrap(),
      // https://www.twitch.tv/gamesdonequick/video/113837699 (legacy url)
      // https://www.twitch.tv/gamesdonequick/v/113837699 (legacy url)
      Regex::new(r"^https?://www\.twitch\.tv/[^/]+/v(?:ideo)?/(?P<video_id>\d+)").unwrap(),
      // https://player.twitch.tv/?video=v113837699&parent=example.com ("v" is optional)
      Regex::new(r"^https?://player\.twitch\.tv/[^#]*[?&]video=v?(?P<video_id>\d+)").unwrap(),
    ]
  })
}

fn clip_url_patterns() -> &'static [Regex] {
  static CLIP_URL_PATTERNS: OnceLock<[Regex; 2]> = OnceLock::new();
  CLIP_URL_PATTERNS.get_or_init(|| {
    [
      // https://clips.twitch.tv/AmazonianKnottyLapwingSwiftRage
      Regex::new(r"^https?://clips\.twitch\.tv/(?P<slug>[^/?#]+)").unwrap(),
      // https://www.twitch.tv/gamesdonequick/clip/ExuberantMiniatureSandpiperDogFace
      Regex::new(r"^https?://www\.twitch\.tv/[^/]+/clip/(?P<slug>[^/?#]+)").unwrap(),
    ]
  })
}

fn game_streams_url_patterns() -> &'static [Regex] {
  static GAME_STREAMS_URL_PATTERNS: OnceLock<[Regex; 1]> = OnceLock::new();
  GAME_STREAMS_URL_PATTERNS.get_or_init(|| {
    [
      // https://www.twitch.tv/directory/category/diablo-ii-resurrected?tl=hardcore&sort=VIEWER_COUNT
      Regex::new(r"^https?://www\.twitch\.tv/directory/category/(?P<slug>[^/?#]+)(?:[?&#](?:tl=(?P<tl>[^&#]+)|sort=(?P<sort>[^&#]+)|cursor=(?P<cursor>[^&#]+)))*").unwrap(),
    ]
  })
}

pub fn probe(url: &str) -> Option<types::TwitchMatch> {
  if crate::CONFIG.twitch_client_id.is_none() {
    return None;
  }

  for re in clip_url_patterns().iter() {
    if cfg!(debug_assertions) {
      log::info!("re: {:?}", re);
    }
    if let Some(captures) = re.captures(url)
      && let Some(m) = captures.get(1)
    {
      return Some(types::TwitchMatch::Clip(m.as_str().to_string()));
    }
  }

  for re in video_url_patterns().iter() {
    if cfg!(debug_assertions) {
      log::info!("re: {:?}", re);
    }
    if let Some(captures) = re.captures(url)
      && let Some(m) = captures.get(1)
    {
      return Some(types::TwitchMatch::Video(m.as_str().to_string()));
    }
  }

  for re in game_streams_url_patterns().iter() {
    if cfg!(debug_assertions) {
      log::info!("re: {:?}", re);
    }
    if let Some(captures) = re.captures(url) {
      let slug = captures.name("slug").unwrap().as_str().to_string();
      let tl = captures.name("tl").map(|m| m.as_str().to_string());
      let sort = captures.name("sort").map(|m| m.as_str().to_string()).unwrap_or("VIEWER_COUNT".to_string());
      let cursor = captures.name("cursor").map(|m| m.as_str().to_string());
      return Some(types::TwitchMatch::GameStreams(slug, tl, sort, cursor));
    }
  }

  for re in channel_videos_url_patterns().iter() {
    if cfg!(debug_assertions) {
      log::info!("re: {:?}", re);
    }
    if let Some(captures) = re.captures(url) {
      let channel_name = captures.name("channel_name").unwrap().as_str().to_string();
      let filter = captures.name("filter").map(|m| m.as_str().to_string()).unwrap_or("all".to_string());
      let sort = captures.name("sort").map(|m| m.as_str().to_string()).unwrap_or("time".to_string());
      let cursor = captures.name("cursor").map(|m| m.as_str().to_string());
      return Some(types::TwitchMatch::ChannelVideos(channel_name, filter, sort, cursor));
    }
  }

  for re in channel_url_patterns().iter() {
    if cfg!(debug_assertions) {
      log::info!("re: {:?}", re);
    }
    if let Some(captures) = re.captures(url)
      && let Some(m) = captures.get(1)
    {
      return Some(types::TwitchMatch::Channel(m.as_str().to_lowercase()));
    }
  }

  return None;
}

pub async fn resolve(m: types::TwitchMatch) -> Result<Vec<PlaylistItem>, Cow<'static, str>> {
  match m {
    types::TwitchMatch::Channel(channel_name) => {
      if channel_name == "twit" {
        // These guys are responsible for most of the traffic and it is a bit annoying
        // Until I can make this configurable in the config file, this channel will just be blocked like this
        return Err("payment required".into());
      }
      resolve_channel(channel_name).await
    }
    types::TwitchMatch::ChannelVideos(channel_name, filter, sort, cursor) => resolve_channel_videos(channel_name, filter, sort, cursor).await,
    types::TwitchMatch::Video(video_id) => resolve_video(video_id).await,
    types::TwitchMatch::Clip(slug) => resolve_clip(slug).await,
    types::TwitchMatch::GameStreams(game_slug, tl, sort, cursor) => resolve_game_streams(game_slug, tl, sort, cursor).await,
  }
}

async fn resolve_channel(channel_name: String) -> Result<Vec<PlaylistItem>, Cow<'static, str>> {
  // https://www.twitch.tv/directory/game/Perfect%20Dark
  // https://www.twitch.tv/directory/category/perfect-dark-2000
  // https://www.twitch.tv/recaps/annual
  if channel_name == "directory" || channel_name == "recaps" {
    return Err("unsupported channel name".into());
  }

  let request_data = json!({
    "query": include_str!("twitch/channel.gql"),
    "variables": {
      "channelName": channel_name,
      "platform": "web",
      "playerType": "site",
    },
  });

  let client = reqwest::Client::builder().build().expect("build reqwest client");
  let client_id = crate::CONFIG.twitch_client_id.as_ref().unwrap().as_str();
  let response = client
    .post(GRAPHQL_URL)
    .header("Client-ID", client_id)
    .body(serde_json::to_string(&request_data).unwrap())
    .send()
    .await
    .expect("send graphql request");
  let response_status = response.status();
  let response_text = response.text().await.expect("read response data");

  if response_status != StatusCode::OK {
    log::error!("bad response: {} - {:?}", response_status, response_text);
    return Err("received non-200 response from Twitch".into());
  }

  let response_data: types::ChannelResponseData = match serde_json::from_str(response_text.as_str()) {
    Ok(v) => v,
    Err(e) => {
      log::error!("error: {:?}, data: {}", e, response_text);
      return Err("error deserializing data".into());
    }
  };
  if cfg!(debug_assertions) {
    log::info!("response_data: {:?}", response_data);
  }
  if let Some(data) = response_data.data {
    if data.channel.is_none() {
      return Err("channel does not exist".into());
    }
    let channel = data.channel.unwrap();
    if channel.stream.is_none() {
      return Err("channel is not live".into());
    }
    let stream = channel.stream.unwrap();
    if stream.playback_access_token.is_none() {
      return Err("playback_access_token is null".into());
    }
    let token = stream.playback_access_token.unwrap();

    return Ok(vec![PlaylistItem {
      path: format!(
        "https://usher.ttvnw.net/api/channel/hls/{}.m3u8?allow_source=true&allow_audio_only=true&sig={}&token={}",
        channel_name,
        urlencoding::encode(token.signature.as_str()),
        urlencoding::encode(token.value.as_str())
      ),
      name: stream.title.unwrap_or("Untitled".into()),
      description: None,
      artist: channel.display_name,
      genre: stream.game.map(|game| game.display_name),
      date: Some(stream.created_at.replace("T", " ").replace("Z", "")),
      duration: None,
      language: stream.language,
    }]);
  } else if let Some(errors) = response_data.errors {
    return Err(format!("Twitch error: {}", errors.iter().map(|e| e.message.as_str()).collect::<Vec<&str>>().join(", ")).into());
  } else {
    return Err("unknown Twitch error".into());
  }
}

async fn resolve_channel_videos(channel_name: String, filter: String, sort: String, cursor: Option<String>) -> Result<Vec<PlaylistItem>, Cow<'static, str>> {
  let q = json!({
    "query": include_str!("twitch/channel_videos.gql"),
    "variables": {
      "login": channel_name,
      "type": filter_to_broadcast_type(filter.clone()),
      "sort": sort.to_uppercase(),
      "limit": 30,
      "cursor": cursor,
    },
  });
  let request_data = serde_json::to_string(&q).unwrap();

  let client = reqwest::Client::builder().build().expect("build reqwest client");
  let client_id = crate::CONFIG.twitch_client_id.as_ref().unwrap().as_str();
  let response = client.post(GRAPHQL_URL).header("Client-ID", client_id).body(request_data).send().await.expect("send graphql request");
  let response_status = response.status();
  let response_text = response.text().await.expect("read response data");

  if response_status != StatusCode::OK {
    log::error!("bad response: {} - {:?}", response_status, response_text);
    return Err("received non-200 response from Twitch".into());
  }

  let response_data: types::ChannelVideosResponseData = match serde_json::from_str(response_text.as_str()) {
    Ok(v) => v,
    Err(e) => {
      log::error!("error: {:?}, data: {}", e, response_text);
      return Err("error deserializing data".into());
    }
  };
  if cfg!(debug_assertions) {
    log::info!("response_data: {:?}", response_data);
  }
  if let Some(data) = response_data.data
    && let Some(user) = data.user
    && let Some(videos) = user.videos
  {
    let last_cursor = videos.edges.last().map(|edge| edge.cursor.clone()).flatten();

    let mut playlist: Vec<_> = videos
      .edges
      .into_iter()
      .map(|edge| PlaylistItem {
        path: format!("https://www.twitch.tv/videos/{}", edge.node.id.unwrap().as_str()),
        name: edge.node.title.unwrap_or("Untitled".into()),
        description: edge.node.description,
        artist: Some(user.display_name.clone()),
        genre: edge.node.game.map(|game| game.display_name),
        date: Some(edge.node.recorded_at.replace("T", " ").replace("Z", "")),
        duration: Some(parse_duration(edge.node.duration.as_str())),
        language: edge.node.language,
      })
      .collect();

    if videos.page_info.has_next_page
      && let Some(last_cursor) = last_cursor
    {
      playlist.push(PlaylistItem {
        path: format!("https://www.twitch.tv/{}/videos?filter={}&sort={}&cursor={}", channel_name, filter, sort, last_cursor),
        name: "Load more".into(),
        description: None,
        artist: Some(user.display_name.clone()),
        genre: None,
        date: None,
        duration: None,
        language: None,
      })
    }

    return Ok(playlist);
  } else if let Some(errors) = response_data.errors {
    return Err(format!("Twitch error: {}", errors.iter().map(|e| e.message.as_str()).collect::<Vec<&str>>().join(", ")).into());
  } else {
    return Err("unknown Twitch error".into());
  }
}

async fn resolve_video(video_id: String) -> Result<Vec<PlaylistItem>, Cow<'static, str>> {
  let q = json!({
    "query": include_str!("twitch/video.gql"),
    "variables": {
      "vodID": video_id,
      "platform": "web",
      "playerType": "site",
    },
  });
  let request_data = serde_json::to_string(&q).unwrap();

  let client = reqwest::Client::builder().build().expect("build reqwest client");
  let client_id = crate::CONFIG.twitch_client_id.as_ref().unwrap().as_str();
  let response = client.post(GRAPHQL_URL).header("Client-ID", client_id).body(request_data).send().await.expect("send graphql request");
  let response_status = response.status();
  let response_text = response.text().await.expect("read response data");

  if response_status != StatusCode::OK {
    log::error!("bad response: {} - {:?}", response_status, response_text);
    return Err("received non-200 response from Twitch".into());
  }

  let response_data: types::VideoResponseData = match serde_json::from_str(response_text.as_str()) {
    Ok(v) => v,
    Err(e) => {
      log::error!("error: {:?}, data: {}", e, response_text);
      return Err("error deserializing data".into());
    }
  };
  if cfg!(debug_assertions) {
    log::info!("response_data: {:?}", response_data);
  }
  if let Some(data) = response_data.data {
    if data.video.is_none() {
      return Err("video is null".into());
    }
    let video = data.video.unwrap();
    if video.playback_access_token.is_none() {
      return Err("playback_access_token is null".into());
    }
    let token = video.playback_access_token.unwrap();

    return Ok(vec![PlaylistItem {
      path: format!(
        "https://usher.ttvnw.net/vod/{}.m3u8?allow_source=true&allow_audio_only=true&sig={}&token={}",
        video_id,
        urlencoding::encode(token.signature.as_str()),
        urlencoding::encode(token.value.as_str())
      ),
      name: video.title.unwrap_or("Untitled".into()),
      description: video.description,
      artist: video.owner.map(|owner| owner.display_name),
      genre: video.game.map(|game| game.display_name),
      date: Some(video.recorded_at.replace("T", " ").replace("Z", "")),
      duration: Some(parse_duration(video.duration.as_str())),
      language: video.language,
    }]);
  } else if let Some(errors) = response_data.errors {
    return Err(format!("Twitch error: {}", errors.iter().map(|e| e.message.as_str()).collect::<Vec<&str>>().join(", ")).into());
  } else {
    return Err("unknown Twitch error".into());
  }
}

async fn resolve_clip(slug: String) -> Result<Vec<PlaylistItem>, Cow<'static, str>> {
  let q = json!({
    "query": include_str!("twitch/clip.gql"),
    "variables": {
      "slug": slug,
      "platform": "web",
      "playerType": "site",
    },
  });
  let request_data = serde_json::to_string(&q).unwrap();

  let client = reqwest::Client::builder().build().expect("build reqwest client");
  let client_id = crate::CONFIG.twitch_client_id.as_ref().unwrap().as_str();
  let response = client.post(GRAPHQL_URL).header("Client-ID", client_id).body(request_data).send().await.expect("send graphql request");
  let response_status = response.status();
  let response_text = response.text().await.expect("read response data");

  if response_status != StatusCode::OK {
    log::error!("bad response: {} - {:?}", response_status, response_text);
    return Err("received non-200 response from Twitch".into());
  }

  let response_data: types::ClipResponseData = match serde_json::from_str(response_text.as_str()) {
    Ok(v) => v,
    Err(e) => {
      log::error!("error: {:?}, data: {}", e, response_text);
      return Err("error deserializing data".into());
    }
  };
  if cfg!(debug_assertions) {
    log::info!("response_data: {:?}", response_data);
  }
  if let Some(data) = response_data.data {
    if data.clip.is_none() {
      return Err("clip is null".into());
    }
    let clip = data.clip.unwrap();
    if clip.playback_access_token.is_none() {
      return Err("playback_access_token is null".into());
    }
    let token = clip.playback_access_token.unwrap();

    let token_value: types::ClipTokenValue = match serde_json::from_str(token.value.as_str()) {
      Ok(v) => v,
      Err(e) => {
        log::error!("error: {:?}", e);
        return Err("error deserializing token_value".into());
      }
    };
    if cfg!(debug_assertions) {
      log::info!("token_value: {:?}", token_value);
    }

    return Ok(vec![PlaylistItem {
      path: format!(
        "{}?allow_source=true&allow_audio_only=true&sig={}&token={}",
        token_value.clip_uri,
        urlencoding::encode(token.signature.as_str()),
        urlencoding::encode(token.value.as_str())
      ),
      name: clip.title.unwrap_or("Untitled".into()),
      description: None,
      artist: Some(clip.broadcaster.display_name),
      genre: clip.game.map(|game| game.display_name),
      date: Some(clip.created_at.replace("T", " ").replace("Z", "")),
      duration: Some(clip.duration_seconds),
      language: clip.language,
    }]);
  } else if let Some(errors) = response_data.errors {
    return Err(format!("Twitch error: {}", errors.iter().map(|e| e.message.as_str()).collect::<Vec<&str>>().join(", ")).into());
  } else {
    return Err("unknown Twitch error".into());
  }
}

async fn resolve_game_streams(game_slug: String, tl: Option<String>, sort: String, cursor: Option<String>) -> Result<Vec<PlaylistItem>, Cow<'static, str>> {
  let q = json!({
    "query": include_str!("twitch/game_streams.gql"),
    "variables": {
      "slug": game_slug,
      "limit": 30,
      "options": {
        "sort": sort,
        "freeformTags": tl,
      },
      "cursor": cursor,
    },
  });
  let request_data = serde_json::to_string(&q).unwrap();

  let client = reqwest::Client::builder().build().expect("build reqwest client");
  let client_id = crate::CONFIG.twitch_client_id.as_ref().unwrap().as_str();
  let response = client.post(GRAPHQL_URL).header("Client-ID", client_id).body(request_data).send().await.expect("send graphql request");
  let response_status = response.status();
  let response_text = response.text().await.expect("read response data");

  if response_status != StatusCode::OK {
    log::error!("bad response: {} - {:?}", response_status, response_text);
    return Err("received non-200 response from Twitch".into());
  }

  let response_data: types::GameStreamsResponseData = match serde_json::from_str(response_text.as_str()) {
    Ok(v) => v,
    Err(e) => {
      log::error!("error: {:?}, data: {}", e, response_text);
      return Err("error deserializing data".into());
    }
  };
  if cfg!(debug_assertions) {
    log::info!("response_data: {:?}", response_data);
  }
  if let Some(data) = response_data.data
    && let Some(game) = data.game
    && let Some(streams) = game.streams
  {
    let last_cursor = streams.edges.last().map(|edge| edge.cursor.clone()).flatten();

    let mut playlist: Vec<_> = streams
      .edges
      .into_iter()
      .map(|edge| PlaylistItem {
        path: format!(
          "https://www.twitch.tv/{}",
          edge.node.broadcaster.as_ref().map(|user| user.login.clone()).flatten().unwrap_or("error".into())
        ),
        name: edge.node.title.unwrap_or("Untitled".into()),
        description: edge.node.viewers_count.map(|viewers_count| format!("{} viewer{}", viewers_count, utils::pluralize(viewers_count))),
        artist: edge.node.broadcaster.map(|user| user.display_name.clone()),
        genre: Some(game.display_name.clone()),
        date: Some(edge.node.created_at.replace("T", " ").replace("Z", "")),
        duration: None,
        language: edge.node.language,
      })
      .collect();

    if streams.page_info.has_next_page
      && let Some(last_cursor) = last_cursor
    {
      playlist.push(PlaylistItem {
        path: format!(
          "https://www.twitch.tv/directory/category/{}?{}sort={}&cursor={}",
          game_slug,
          tl.map(|tl| format!("tl={}&", tl)).unwrap_or("".to_string()),
          sort,
          last_cursor
        ),
        name: "Load more".into(),
        description: None,
        artist: None,
        genre: Some(game.display_name),
        date: None,
        duration: None,
        language: None,
      })
    }

    return Ok(playlist);
  } else if let Some(errors) = response_data.errors {
    return Err(format!("Twitch error: {}", errors.iter().map(|e| e.message.as_str()).collect::<Vec<&str>>().join(", ")).into());
  } else {
    return Err("unknown Twitch error".into());
  }
}

// all => None, archives => ARCHIVE, highlights => HIGHLIGHT, uploads => UPLOAD
// TODO: Add validation
fn filter_to_broadcast_type(filter: String) -> Option<String> {
  if filter == "all" {
    return None;
  }
  let mut broadcast_type = filter.to_uppercase();
  broadcast_type.pop();
  return Some(broadcast_type);
}

fn parse_duration(s: &str) -> usize {
  let mut seconds = 0;
  let mut numbers = String::with_capacity(2);
  for c in s.chars() {
    if c.is_ascii_digit() {
      numbers.push(c);
    } else if c == 'h' || c == 'm' || c == 's' {
      match numbers.parse::<usize>() {
        Ok(n) => {
          if n != 0 {
            if c == 'h' {
              seconds += 3600 * n;
            } else if c == 'm' {
              seconds += 60 * n;
            } else if c == 's' {
              seconds += n;
            }
          }
        }
        Err(e) => {
          log::error!("parse_duration({}) error: {}", s, e);
        }
      }
      numbers.clear();
    } else {
      log::error!("parse_duration({}) unexpected character: {}", s, c);
    }
  }
  return seconds;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test() {
    // Durations taken from Twitch:
    assert_eq!(parse_duration("32h47m50s"), 118070);
    assert_eq!(parse_duration("1h20m0s"), 4800);
    assert_eq!(parse_duration("55m31s"), 3331);
    assert_eq!(parse_duration("2m53s"), 173);
    assert_eq!(parse_duration("58s"), 58);

    // Hypothetical durations that shouldn't error:
    assert_eq!(parse_duration("1h"), 3600);
    assert_eq!(parse_duration("1m"), 60);
    assert_eq!(parse_duration("0s"), 0);

    // Not seen on Twitch so they don't work properly:
    assert_eq!(parse_duration("1d8h47m50s"), 67670);
    assert_eq!(parse_duration("1y10d"), 0);
  }
}
