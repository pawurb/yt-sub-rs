# yt-sub-rs [![Latest Version](https://img.shields.io/crates/v/yt-sub.svg)](https://crates.io/crates/yt-sub) [![GH Actions](https://github.com/pawurb/yt-sub-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/pawurb/yt-sub-rs/actions)

yt-sub is a simple CLI for subscribing to YouTube RSS feeds without using a YouTube account, powered by [CloudFlare Workers](https://workers.cloudflare.com/) backend. Receive Slack notifications about fresh videos based on the configurable schedule.

## Usage

```text
YouTube RSS subscription CLI

Usage: ytsub <COMMAND>

Commands:
  init          Initialize config file [aliases: i]
  settings      Display current settings [aliases: s]
  run           Check and notify about fresh videos [aliases: r]
  channel-data  Get a channel data based on its handle [aliases: d]
  follow        Subscribe to a channel [aliases: f]
  unfollow      Unsubscribe [aliases: u]
  list          List followed channels [aliases: l]
  register      Register remote account [aliases: re]
  unregister    Remove remote account [aliases: un]
  sync          Update remote settings to match local [aliases: sc]
  help          Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Install CLI:

```bash
cargo install yt-sub
```

Initialize the settings file at `~/.config/yt-sub-rs/config.toml`:

```bash
ytsub init
```

`~/.config/yt-sub-rs/config.toml`
```toml
channels = []

[[notifiers]]
Log = []
```

Follow preferred channels based on [their URL handle](https://support.google.com/youtube/answer/11585688?sjid=4360661106629707574-EU):

```bash
ytsub follow --handle @ManofRecaps
```

Display the list of your channels by typing:

```bash
ytsub list

# You are following:
#
# name: Man of Recaps
# handle: @ManofRecaps
# channel_id: UCNCTxLZ3EKKry-oWgLlsYsw
# channel_url: https://www.youtube.com/@ManofRecaps
# RSS feed: https://www.youtube.com/feeds/videos.xml?channel_id=UCNCTxLZ3EKKry-oWgLlsYsw

```

Now you can run: 

```bash
ytsub run

# New video - Man of Recaps Rings of Power RECAP: Season 2 https://www.youtube.com/watch?v=CjeUx_HHtF0
```

to trigger notifications about freshly released videos from your observed channels. The first `run` invocation will notify you about videos released in the last 7 days. Subsequent runs will inform about videos released since the previous `run` event.

You can customize this period by appending the `--hours-offset` option:

```bash
# notify about videos published in the last 24 hours
ytsub run --hours-offset 24 
```

The time of the last run is stored in the `~/.yt-sub-rs/last_run_at.txt` file.

You can unfollow a channel by typing:

```bash
ytsub unfollow --handle @ManofRecaps
```

## Remote account

By registering a remote account, you'll receive Slack notifications about new videos without executting the CLI locally.

To do it run:

```bash
ytsub register
```

it will persist your API key in the config file. You must configure Slack notifications to use this feature. 

If your account is registered `follow` and `unfollow` commands will automatically update your remote data. If you change the config file manually, you have to run:

```bash
ytsub sync
```

By default, new videos are checked once every hour. Optionally, you can define a notification schedule like this:

`~/.config/yt-sub-rs/config.toml`

```toml
schedule = [8, 20]
```

Numbers represent UTC hours when notifications should be sent.

Don't forget to run `sync` after updating this config.

You can remove all your remote data by running:

```bash
ytsub unregister
```

## Notifiers configuration

By default, CLI is configured to log to the `stdout`. You can configure Slack notifications like this:

`~/.config/yt-sub-rs/config.toml`

```toml
[[notifiers]]

[notifiers.Slack]
webhook_url = "https://hooks.slack.com/services/XXX/XXX/XXX"
channel = "yt-videos"
```

![Slack notification](https://github.com/pawurb/yt-sub-rs/raw/main/assets/slack-notification.png)

You can obtain the `webhook_url` value as described [in the Slack docs](https://api.slack.com/messaging/webhooks).

## Manually finding an RSS `channel_id`

CLI will try to find the matching `channel_id` based on the URL handle. But proxied YouTube API calls are sometimes throttled. So if the `follow` command fails, you have to obtain this data manually. Go to the [channel videos tab](https://www.youtube.com/@ManofRecaps/videos) and run this JS in the console to extract the RSS `channel_id`:

```javascript
document.querySelector('link[rel="alternate"][type="application/rss+xml"]').href.match(/channel_id=([^&]+)/)[1];
```
This snippet works on Firefox. Alternatively, you can look for this value manually in the source code: 

![RSS feed](https://github.com/pawurb/yt-sub-rs/raw/main/assets/youtube-rss.png)

Now you can subscribe by providing all the data like this:

```bash
ytsub follow --handle @ManofRecaps --channel-id UCNCTxLZ3EKKry-oWgLlsYsw --desc 'Man of Recaps'
```


## CRON invocation

Alternatively, you can use the CLI via the CRON scheduler to receive notifications without a remote account. By appending the `--cron` flag to the `run` command CLI will output the logs with timestamps:

```
RUST_LOG=info ytsub run --cron

# [2024-10-14T13:03:58Z INFO  yt_sub::logger] New video - Man of Recaps Rings of Power RECAP: Season 2 https://www.youtube.com/watch?v=CjeUx_HHtF0
```

This mode of execution is the most useful with Slack notifications configured.

## Status

This project is in the early stages of development, so feedback and PRs are welcome.
