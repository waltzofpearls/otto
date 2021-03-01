<img align="right" width="160" src="https://user-images.githubusercontent.com/965430/109471904-4b8fe700-7a26-11eb-8b65-228b2ac5910a.png">

# otto

[![Build Status][actions-badge]][actions-url]
[![MIT licensed][mit-badge]][mit-url]

[actions-badge]: https://github.com/waltzofpearls/otto/workflows/ci/badge.svg
[actions-url]: https://github.com/waltzofpearls/otto/actions?query=workflow%3Aci+branch%3Amain
[mit-badge]: https://img.shields.io/badge/license-Apache%202-blue.svg
[mit-url]: https://github.com/waltzofpearls/otto/blob/main/LICENSE

Yet another monitoring tool that collects results and then alert on failures except... this one
was specifically designed to watch external service providers. It helps answer the question of
"is our API down, or is AWS/Cloudfare/Heroku down?", when outages occurred.

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

Otto is equipped with **Probe** plugins:

- Atom feed
- RSS feed
- HTTP
- Exec (shell scripts)

**Alert** plugins:

- Slack
- Email (SMTP)
- Webhook

And `/metrics` endpoint for promethues.

Plugins are [configurable](./examples/README.md#configure-it), and can be reloaded by `kill -SIGHUP $(pidof otto)`.

### Try it

Copy `simple.toml` from `examples` folder, and rename it to `otto.toml`.

```shell
cp examples/simple.toml otto.toml
```

Open `otto.toml` with an editor. In `alerts.slack`, replace the fake url with an actual webhook url copied from Slack.

If you have rust installed on your computer, run otto locally with `make run`, it needs `rustc` and `cargo`, or run it
with `make docker`, if your machine has docker installed.

### Known issues

In alpine linux, firing slack and email alerts will trigger segmentation fault. This issue might be caused by calling
slack-hook and lettre libs inside async runtime. It's possbile the segfault is related to musl libc.
