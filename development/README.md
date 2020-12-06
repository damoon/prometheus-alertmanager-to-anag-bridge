_alertmanager API_

https://github.com/prometheus/alertmanager/blob/master/api/v2/openapi.yaml

`curl -H "Content-Type: application/json" 127.0.0.1:9093/api/v2/alerts | jq .`

_nagios output_

http://alerting/cgi-bin/icinga/status.cgi
    ?style=servicedetail
    &embedded
    &limit=0
    &serviceprops=262144 # = hard problems 524288=soft
    &servicestatustypes=61
    &jsonoutput

servicestatustypes 61 = 32 + 16 + 8 + 4 + 1
 1 Pending
 2 Ok
 4 Warning
 8 Unknown
16 Critical

http://alerting/cgi-bin/icinga/status.cgi
    ?limit=0
    &style=hostdetail
    &hostprops=0
    &hoststatustypes=12
    &jsonoutput

hoststatustypes 12 = 8 + 4
1 Pending
2 Up
4 Down
8 Unreachable


http://alerting/cgi-bin/icinga/tac.cgi
    ?jsonoutput
