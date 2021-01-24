# otto

[![Build Status][actions-badge]][actions-url]
[![MIT licensed][mit-badge]][mit-url]

[actions-badge]: https://github.com/waltzofpearls/otto/workflows/ci/badge.svg
[actions-url]: https://github.com/waltzofpearls/otto/actions?query=workflow%3Aci+branch%3Amain
[mit-badge]: https://img.shields.io/badge/license-Apache%202-blue.svg
[mit-url]: https://github.com/waltzofpearls/otto/blob/main/LICENSE

It's not just another monitoring tool that collects results and then alert on failures.
Otto was initially designed to solve a specific problem, when outage happens, effortlessly
answer the question of "is our API down, or is AWS down?", and of cource AWS here is
interchangeable with other service providers.

Otto is good at:

- Watch status page
 - Read and parse Atom or RSS feed
 - [AWS][aws-status], [Cloudflare][cloudflare-status], [GitHub][github-status],
   [Heroku][heroku-status] and many more
- Test URL liveness
 - Check response code from an API or a web page
- Periodic execution
 - Execute shell script on schedule and evaluate exit code

[aws-status]: https://status.aws.amazon.com/
[cloudflare-status]: https://www.cloudflarestatus.com/
[github-status]: https://www.githubstatus.com/
[heroku-status]: https://status.heroku.com/

agent that probes various sources and then alert on failures.

Otto is equipped with **Probe** plugins:

- Atom
- RSS
- HTTP
- Exec (shell scripts)

And **Alert** plugins:

- Slack
- Email (SMTP)
- [WIP] Webhook
- [WIP] Prometheus metrics

Plugins are configurable through a TOML config file:

```toml
schedule = "0 * * * * *"
log_level = "info"

[[probes.exec]]
schedule = "0/30 * * * * *"
cmd = "./test.sh"

[[probes.http]]
schedule = "0/30 * * * * *"
url = "https://google.ca"
method = "get"
expected_code = 200

[[probes.http]]
schedule = "0/50 * * * * *"
url = "https://httpbin.org/post"
method = "post"
headers = { Content-Type = "application/json" }
json = """{
    "key1": "value1",
    "key2": "value2"
}"""
expected_code = 200

[[probes.atom]]
schedule = "0 * * * * *"
feed_url = "https://www.cloudflarestatus.com/history.atom"
# Content should contain Investigating and not contain Resolved
# for regex lookahead and negative look ahead see the following stack overflow answer
# https://stackoverflow.com/questions/8240765/is-there-a-regex-to-match-a-string-that-contains-a-but-does-not-contain-b
content_regex = "^(?=.*Investigating)(?!.*Resolved).*"

[[probes.rss]]
schedule = "0 * * * * *"
feed_url = "https://www.githubstatus.com/history.rss"
# Content should contain Investigating and not contain Resolved
# for regex lookahead and negative look ahead see the following stack overflow answer
# https://stackoverflow.com/questions/8240765/is-there-a-regex-to-match-a-string-that-contains-a-but-does-not-contain-b
description_regex = "^(?=.*Investigating)(?!.*Resolved).*"

[[alerts.slack]]
webhook_url = "https://hooks.slack.com/services/abc/123/45z"

[[alerts.email]]
smtp_relay = "smtp.gmail.com"
smtp_username = "some.username@gmail.com"
smtp_password = "some.app.password"
from = "Otto <otto@ottobot.io>"
to = "Joe Smith <joe@smith.com>"
```
