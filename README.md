VLC Playlist Parser Proxy... [because VLC can't POST 🤦‍♂️](https://code.videolan.org/videolan/vlc/-/issues/26185)

Run locally:

```shell
export TWITCH_CLIENT_ID=youcanfindthisonline
cargo run
```

Resolve a URL to a media endpoint:

```shell
$ curl -v http://localhost:8080/resolve?url=https://www.twitch.tv/videos/113837699

[...]

< HTTP/1.1 307 Temporary Redirect
< location: https://usher.ttvnw.net/vod/113837699.m3u8?[.......]
```

Return the data in JSON:

```shell
$ curl -sSf 'http://localhost:8080/resolve?url=https://www.twitch.tv/videos/113837699&output=json' | jq
[
  {
    "path": "https://usher.ttvnw.net/vod/113837699.m3u8?[.......]",
    "name": "AGDQ 2017 benefitting the Prevent Cancer Foundation - Mickey's Dangerous Chase",
    "description": null,
    "language": "en",
    "artist": "GamesDoneQuick",
    "genre": "System Shock 2",
    "date": "2017-01-10 20:10:16",
    "duration": 118070
  }
]
```

The keys were chosen based on what VLC supports. More may be added or removed in future versions.
