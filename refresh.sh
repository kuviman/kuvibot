#!/usr/bin/env bash

set -eu

# Issue an access token for Twitch
[[ -f .secret/.client_id ]]     && export CLIENT_ID="$(cat .secret/.client_id)"
[[ -f .secret/.client_secret ]] && export CLIENT_SECRET="$(cat .secret/.client_secret)"
[[ -f .secret/.refresh_token ]] && export REFRESH_TOKEN="$(cat .secret/.refresh_token)"
[[ -f .secret/.access_token ]]  && export ACCESS_TOKEN="$(cat .secret/.access_token)"

RESPONSE=$(curl -Ss -X POST "https://id.twitch.tv/oauth2/token" \
    -F "client_id=${CLIENT_ID}" \
    -F "refresh_token=${REFRESH_TOKEN}" \
    -F "client_secret=${CLIENT_SECRET}" \
    -F 'grant_type=refresh_token' )

REFRESH_TOKEN=$(echo "$RESPONSE" | jq -r '.refresh_token')
ACCESS_TOKEN=$(echo "$RESPONSE" | jq -r '.access_token')

if [[ "$ACCESS_TOKEN" != "null" ]]; then
  echo "$ACCESS_TOKEN" > .secret/.access_token
fi
if [[ "$REFRESH_TOKEN" != "null" ]]; then
  echo "$REFRESH_TOKEN" > .secret/.refresh_token
fi
