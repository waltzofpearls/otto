# Examples

### Configure it

Otto is configured by a combination of command line options and TOML config file.

Command line options can specify path to config file and log level filter:

- `-c, --config`: path to TOML config file
  - default value: `/etc/otto/otto.toml`
  - example: `otto -c ./examples/simple.toml`
- `-l, --log-level`: log level filter
  - default value: `info`
  - more options: `error`, `warn`, `debug` and `trace`

A complete config file is consisted of global configs, probe plugins and alert plugins.

#### Global configs

```toml
schedule = "0 * * * * *"

[prometheus]
listen = "127.0.0.1:9999"
path = "metrics"
```

#### Probe plugins

Exec

```toml
[[probes.exec]]
# Check github.com's SSL cert, and alert when it's expired
cmd = "./examples/check_ssl_cert.sh"
args = ["github.com"]
```

HTTP

```toml
[[probes.http]]
# Send GET request to google.ca every 30 seconds, and alert when response status code is not 200
schedule = "0/30 * * * * *"
url = "https://google.ca"
method = "get"
expected_code = 200

[[probes.http]]
# Post JSON to httpbin.org once every 50 seconds, and alert when response status code is not 200
schedule = "0/50 * * * * *"
url = "https://httpbin.org/post"
method = "post"
headers = { Content-Type = "application/json" }
json = """{
    "key1": "value1",
    "key2": "value2"
}"""
expected_code = 200
```

Atom feed

```toml
[[probes.atom]]
# Watch Heroku's status Atom feed for posted incidents
feed_url = "https://feeds.feedburner.com/herokustatus"
title_regex = "^(?!.*Resolved).*"

[[probes.atom]]
# Watch Cloudflare's status Atom feed for posted incidents
feed_url = "https://www.cloudflarestatus.com/history.atom"
# Content should contain Investigating and not contain Resolved
# for regex lookahead and negative look ahead see the following stack overflow answer
# https://stackoverflow.com/questions/8240765/is-there-a-regex-to-match-a-string-that-contains-a-but-does-not-contain-b
content_regex = "^(?=.*Investigating)(?!.*Resolved).*"
```

RSS feed

```toml
[[probes.rss]]
# Watch github's status RSS feed for posted incidents
feed_url = "https://www.githubstatus.com/history.rss"
# Content should contain Investigating and not contain Resolved
# for regex lookahead and negative look ahead see the following stack overflow answer
# https://stackoverflow.com/questions/8240765/is-there-a-regex-to-match-a-string-that-contains-a-but-does-not-contain-b
description_regex = "^(?=.*Investigating)(?!.*Resolved).*"
```

#### Alert plugins

Slack

```toml
[[alerts.slack]]
# Send alert to Slack's webhook url
webhook_url = "https://hooks.slack.com/services/abc/123/45z"
```

Email

```toml
[[alerts.email]]
# Send alert to an email address via Gmail
smtp_relay = "smtp.gmail.com"
smtp_username = "some.username@gmail.com"
smtp_password = "some.app.password"
from = "Otto <otto@ottobot.io>"
to = "Joe Smith <joe@smith.com>"
```

Webhook

```toml
[[alerts.webhook]]
# Post alert to a webhook url
url = "https://httpbin.org/post"
headers = { Content-Type = "application/json" }
```

More examples:

- [`simple.toml`](./simple.toml): one probe and one alert
- [`fancy.toml`](./fancy.toml): many probes and many alerts
