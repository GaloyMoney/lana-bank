## general

for webhook (opt):

on one terminal:
`python3 webhook.py`

on another one, launch tmole:
`npx tmole 5253`

the webhook needs to now be updated from sumsub interface here
https://cockpit.sumsub.com/checkus#/devSpace/webhooks/webhookManager

it won't work with staging, so the staging webhook needs to be deactivated if errors arise


if tmole give this:  https://spylwl-ip-12-123-12-123.tunnelmole.net -> http://localhost:5253 

add `https://spylwl-ip-12-123-12-123.tunnelmole.net/webhook/sumsub` to sumsub callback api

## for applicant

1. create a new link with ./create_sumsub_link.sh

.env need to be configured with SUMSUB_KEY and SUMSUB_SECRET

to test it works correctly locally, run 

```
curl -X POST http://localhost:5253/sumsub/callback \
  -H "Content-Type: application/json" \
  -d '{
    "applicantId": "test-applicant-id",
    "externalUserId": "test-user-id",
    "type": "applicantCreated",
    "reviewStatus": "init"
  }'
```

2. calling sumsub for the result 

`./get_sumsub_applicant.sh $(cat .sumsub_customer_id)`


## for transactions

`./submit_finance_transaction.sh`
