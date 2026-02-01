use serde::Deserialize;

#[derive(Debug)]
pub enum TwitchMatch {
  Channel(String),
  ChannelVideos(String, String, String, Option<String>),
  Video(String),
  Clip(String),
}

// Channel
#[derive(Debug, Deserialize)]
pub(super) struct ChannelResponseData {
  pub data: ChannelData,
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
  pub title: String,
  pub created_at: String,
  pub language: String,
  pub game: Option<Game>,
  pub playback_access_token: PlaybackAccessToken,
}

// ChannelVideos
#[derive(Debug, Deserialize)]
pub(super) struct ChannelVideosResponseData {
  pub data: ChannelVideosData,
}

#[derive(Debug, Deserialize)]
pub(super) struct ChannelVideosData {
  pub user: Option<UserWithVideos>,
}

// Video
#[derive(Debug, Deserialize)]
pub(super) struct VideoResponseData {
  pub data: VideoData,
}

#[derive(Debug, Deserialize)]
pub(super) struct VideoData {
  pub video: Option<Video>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Video {
  pub id: Option<String>,
  pub title: String,
  pub description: Option<String>,
  pub owner: Option<User>,
  pub game: Option<Game>,
  pub recorded_at: String,
  pub duration: String,
  pub language: String,
  pub playback_access_token: Option<PlaybackAccessToken>,
}

// Clip
#[derive(Debug, Deserialize)]
pub(super) struct ClipResponseData {
  pub data: ClipData,
}

#[derive(Debug, Deserialize)]
pub(super) struct ClipData {
  pub clip: Option<Clip>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Clip {
  pub title: String,
  pub broadcaster: User,
  pub game: Option<Game>,
  pub created_at: String,
  pub duration_seconds: usize,
  pub language: String,
  pub playback_access_token: PlaybackAccessToken,
}

#[derive(Debug, Deserialize)]
pub(super) struct ClipTokenValue {
  pub clip_uri: String,
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
  pub display_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UserWithVideos {
  pub display_name: String,
  pub videos: VideoConnection,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct VideoConnection {
  pub edges: Vec<VideoEdge>,
  pub page_info: PageInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct VideoEdge {
  pub cursor: String,
  pub node: Video,
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
