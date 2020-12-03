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

Setup [Rust](https://rustup.rs/) with your editor of choice.

Setup [Tilt](https://docs.tilt.dev/install.html).

Run tilt to deploy patam and its dependencies.
``` bash
tilt up
```

Install [Anbox](https://docs.anbox.io/userguide/install.html) to run Android apps locally.

Download [Anag](https://apkpure.com/de/anag/info.degois.damien.android.aNag) and [add](https://docs.anbox.io/userguide/install_apps.html) it to Anbox.

Configure Anag to the endpoint of patam.

__Endpoints__

Papam: TODO: http://{host-ip}:8080/nagios/cgi-bin/

Prometheus: [http://localhost:9090/alerts](http://localhost:9090/alerts)

Alertmanager: [http://localhost:9093/#/alerts](http://localhost:9093/#/alerts)
