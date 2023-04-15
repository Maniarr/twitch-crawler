# Twitch Crawler

Worker used to collect viewers count from twitch streams and transmit them to the [Warp10 database](https://www.warp10.io/).

## Metrics

Metrics collected:

- `<prefix>.viewers`

Metric labels available:

- `event_name`
- `stream_id`
- `game_id`
- `game_name`
- `user_id`
- `user_name`


## Build

Rust version: 1.68

```sh
cargo build
```

## Usage

### CLI

Parameters can be passed to binary by environment variables or as command parameters.

```sh
twitch-crawler -h

Usage: twitch-crawler [OPTIONS] --event-name <EVENT_NAME> --twitch-client-id <TWITCH_CLIENT_ID> --twitch-client-secret <TWITCH_CLIENT_SECRET> --warp10-url <WARP10_URL> --warp10-write-token <WARP10_WRITE_TOKEN>

Options:
      --event-name <EVENT_NAME>
          Value of Warp10 label "event_name" of datapoints [env: EVENT_NAME=]
      --twitch-client-id <TWITCH_CLIENT_ID>
          Twitch client id [env: TWITCH_CLIENT_ID=]
      --twitch-client-secret <TWITCH_CLIENT_SECRET>
          Twitch client secret [env: TWITCH_CLIENT_SECRET=]
      --warp10-url <WARP10_URL>
          Base url of Warp10 database [env: WARP10_URL=]
      --warp10-write-token <WARP10_WRITE_TOKEN>
          Warp10 write token [env: WARP10_WRITE_TOKEN=]
      --warp10-prefix <WARP10_PREFIX>
          Warp10 classname prefix of datapoints [env: WARP10_PREFIX=] [default: twitch]
      --game-ids <GAME_IDS>
          Filter streams on game ids (Max value: 100) [env: GAME_IDS=]
      --languages <LANGUAGES>
          Filter streams on languages (Max value: 100) [env: LANGUAGES=]
      --user-logins <USER_LOGINS>
          Filter streams on user logins [env: USER_LOGINS=]
      --minimum-viewers <MINIMUM_VIEWERS>
          Keep only datapoint viewers count superior to the value [env: MINIMUM_VIEWERS=] [default: 0]
      --interval <INTERVAL>
          Interval of seconds between each measurement [env: INTERVAL=] [default: 15]
  -h, --help
          Print help information
  -V, --version
          Print version information
```

### Docker

By default, Warp10 ports are forwarded to 127.0.0.1 to access to Warp10 studio & warp10 endpoint to show metrics.

- Warp10 endpoint port: `8080`
- Warp10 Studio port: `8081`

Launch first Warp10 database, in order to retrieve warp10 write token available in container logs at the first launch:

```sh
docker-compose up -d warp10
```

Launch twitch-crawler:

```sh
export TWITCH_CLIENT_ID=xxx 
export TWITCH_CLIENT_SECRET=xxx 
export WARP10_WRITE_TOKEN=xxx 

docker-compose up -d twitch-crawler
```
