# Prometheus alertmanager to Anag bridge

PATAB brides the connection between [Anag](https://damien.degois.info/android/aNag/) (Android client for Nagios) as client to the [alertmanager](https://prometheus.io/docs/alerting/latest/alertmanager/) of Prometheus.

It pretends to be a nagios endpoint with json encoding.

Supported:
- List triggering alerts
- Acknowledge alerts

Unsupported:
- Recheck alerts
- Filters for services
- Hosts
- Generally everything nagios does but alertmanager does not

## Development

Setup [Rust](https://rustup.rs/).

Install [Anbox](https://docs.anbox.io/userguide/install.html) to run Android apps locally.

Download [Anag](https://apkpure.com/de/anag/info.degois.damien.android.aNag) and start it in Anbox.

TODO: document patab endpoint for anag

TODO: implement alertmanager mock endpoint or start alertmanager + prometheus in docker

__tip__

Use [cargo-watch](https://github.com/passcod/cargo-watch) to run `cargo watch -x check -x test -x run`

Rustfmt is your friend :D
