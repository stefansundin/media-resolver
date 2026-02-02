use serde::Deserialize;

#[derive(Debug)]
pub enum TwitchMatch {
  Channel(String),
  ChannelVideos(String, String, String, Option<String>),
  Video(String),
  Clip(String),
  GameStreams(String, Option<String>, String, Option<String>),
}

#[derive(Debug, Deserialize)]
pub(super) struct ErrorData {
  pub message: String,
}

// Channel
#[derive(Debug, Deserialize)]
pub(super) struct ChannelResponseData {
  pub data: Option<ChannelData>,
  pub errors: Option<Vec<ErrorData>>,
}

#[derive(Debug, Deserialize)]
pub(super) struct ChannelData {
  pub channel: Option<Channel>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Channel {
  pub display_name: Option<String>,
  pub stream: Option<Stream>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Stream {
  pub title: Option<String>,
  pub created_at: String,
  pub language: Option<String>,
  pub viewers_count: Option<usize>,
  pub game: Option<Game>,
  pub broadcaster: Option<User>,
  pub playback_access_token: Option<PlaybackAccessToken>,
}

// ChannelVideos
#[derive(Debug, Deserialize)]
pub(super) struct ChannelVideosResponseData {
  pub data: Option<ChannelVideosData>,
  pub errors: Option<Vec<ErrorData>>,
}

#[derive(Debug, Deserialize)]
pub(super) struct ChannelVideosData {
  pub user: Option<UserWithVideos>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UserWithVideos {
  pub display_name: String,
  pub videos: Option<VideoConnection>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct VideoConnection {
  pub edges: Vec<Edge<Video>>,
  pub page_info: PageInfo,
}

// Video
#[derive(Debug, Deserialize)]
pub(super) struct VideoResponseData {
  pub data: Option<VideoData>,
  pub errors: Option<Vec<ErrorData>>,
}

#[derive(Debug, Deserialize)]
pub(super) struct VideoData {
  pub video: Option<Video>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Video {
  pub id: Option<String>,
  pub title: Option<String>,
  pub description: Option<String>,
  pub owner: Option<User>,
  pub game: Option<Game>,
  pub recorded_at: String,
  pub duration: String,
  pub language: Option<String>,
  pub playback_access_token: Option<PlaybackAccessToken>,
}

// Clip
#[derive(Debug, Deserialize)]
pub(super) struct ClipResponseData {
  pub data: Option<ClipData>,
  pub errors: Option<Vec<ErrorData>>,
}

#[derive(Debug, Deserialize)]
pub(super) struct ClipData {
  pub clip: Option<Clip>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Clip {
  pub title: Option<String>,
  pub broadcaster: User,
  pub game: Option<Game>,
  pub created_at: String,
  pub duration_seconds: usize,
  pub language: Option<String>,
  pub playback_access_token: Option<PlaybackAccessToken>,
}

#[derive(Debug, Deserialize)]
pub(super) struct ClipTokenValue {
  pub clip_uri: String,
}

// GameStreams
#[derive(Debug, Deserialize)]
pub(super) struct GameStreamsResponseData {
  pub data: Option<GameStreamsData>,
  pub errors: Option<Vec<ErrorData>>,
}

#[derive(Debug, Deserialize)]
pub(super) struct GameStreamsData {
  pub game: Option<GameWithStreams>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GameWithStreams {
  pub display_name: String,
  pub streams: Option<StreamConnection>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StreamConnection {
  pub edges: Vec<Edge<Stream>>,
  pub page_info: PageInfo,
}

// Shared
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Game {
  pub display_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct User {
  pub login: Option<String>,
  pub display_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Edge<T> {
  pub cursor: Option<String>,
  pub node: T,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PageInfo {
  pub has_next_page: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct PlaybackAccessToken {
  pub signature: String,
  pub value: String,
}
