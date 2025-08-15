#!/usr/bin/env zsh
# Simple endpoint smoke-test script using curl + jq
# Usage:
#   BASE_URL=http://127.0.0.1:3000 JWT_SECRET=... ./scripts/test_endpoints.sh
# Requires: jq, curl

set -euo pipefail

if ! command -v curl >/dev/null 2>&1; then
  echo "curl is required"
  exit 1
fi
if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required (install via your package manager)"
  exit 1
fi

BASE_URL=${BASE_URL:-http://127.0.0.1:3000}

# helper that returns body then status on separate lines
# On curl failure (non-zero exit), prints curl error output as body and returns 000 as status.
http() {
  local method=$1; shift
  local url=$1; shift
  local data=${1-}

  # build optional auth header if TOKEN is set
  local auth_args=()
  if [[ -n "${TOKEN-}" ]]; then
    auth_args=( -H "Authorization: Bearer $TOKEN" )
  fi

  local out
  if [[ -n "$data" ]]; then
    # include the status code on the final line
    out=$(curl -sS -w "\n%{http_code}" -X "$method" "$url" -H "Content-Type: application/json" "${auth_args[@]}" -d "$data" 2>&1) || {
      # curl failed; print stderr-like output and a sentinel status 000
      echo "$out"
      echo "000"
      return
    }
  else
    out=$(curl -sS -w "\n%{http_code}" -X "$method" "$url" "${auth_args[@]}" 2>&1) || {
      echo "$out"
      echo "000"
      return
    }
  fi

  # curl succeeded and wrote body then a trailing newline then the HTTP status
  echo "$out"
}

# parse raw response (body + "\n" + status) into globals RESP_BODY and RESP_STATUS
# Works even if body doesn't end with a newline.
parse_response() {
  local raw="$1"
  # Use printf to preserve newlines, then extract last line as status and the rest as body
  RESP_STATUS=$(printf "%s" "$raw" | awk 'END{print $0}')
  # body is everything except the last line
  RESP_BODY=$(printf "%s" "$raw" | awk 'NR==1{line=$0; next} { if(NR==2) {body=line"\n"$0; next} else {body=body"\n"$0}} END{ if(NR<=1) {print ""} else { # print all but last line
    for(i=1;i<NR;i++){ if(i==1) printf "%s", (i==1?lines[i]:lines[i]) }
  }}')
  # Fallback parsing (more portable) if awk above didn't work as intended: split by lines
  if [[ -z "$RESP_STATUS" ]]; then
    # safer: get last line and body via tail/sed
    RESP_STATUS=$(printf "%s" "$raw" | tail -n1)
    RESP_BODY=$(printf "%s" "$raw" | sed '$d')
  fi
}

# Helper that calls http and fills local variables 'body' and 'http_status'
call_and_parse() {
  local method=$1; shift
  local url=$1; shift
  local data=${1-}
  local raw
  if [[ -n "$data" ]]; then
    raw=$(http "$method" "$url" "$data")
  else
    raw=$(http "$method" "$url")
  fi
  parse_response "$raw"
  # Export into variables used by the script
  body="$RESP_BODY"
  http_status="$RESP_STATUS"
}

echo "Using BASE_URL=$BASE_URL"

TS=$(date +%s)
USERNAME="ituser_$TS"
EMAIL="${USERNAME}@example.com"
PASSWORD="${TEST_PASSWORD:-password123}"

# 1) Health check
echo "\n== health check =="
call_and_parse GET "$BASE_URL/health"
echo "status=$http_status body=$body"

# 2) Users CRUD
echo "\n== users: create =="
create_payload=$(jq -n --arg u "$USERNAME" --arg e "$EMAIL" --arg p "$PASSWORD" '{username:$u,email:$e,password:$p}')
call_and_parse POST "$BASE_URL/users" "$create_payload"
echo "status=$http_status body=$body"
if [[ "$http_status" == "200" ]]; then
  USER_ID=$(echo "$body" | jq -r '.id')
elif [[ "$http_status" == "409" ]]; then
  echo "user already exists; will attempt login"
else
  echo "user create failed (status=$http_status)"
  echo "body: $body"
  exit 2
fi

# 3) Auth: login -> token
echo "\n== auth: login =="
login_payload=$(jq -n --arg u "$USERNAME" --arg p "$PASSWORD" '{username:$u,password:$p}')
call_and_parse POST "$BASE_URL/auth/login" "$login_payload"
echo "status=$http_status body=$body"
if [[ "$http_status" != "200" ]]; then
  echo "login failed"
  exit 3
fi
TOKEN=$(echo "$body" | jq -r '.token')

# 4) Auth: whoami
echo "\n== auth: whoami =="
call_and_parse GET "$BASE_URL/auth/whoami"
echo "status=$http_status body=$body"

# 5) Create blog
echo "\n== blog: create =="
BLOG_SLUG="blog-$TS"
blog_payload=$(jq -n --arg t "Hello $TS" --arg s "$BLOG_SLUG" --arg b "body" --arg a "$USERNAME" '{title:$t,slug:$s,body:$b,author:$a}')
call_and_parse POST "$BASE_URL/communication/blog" "$blog_payload"
echo "status=$http_status body=$body"
if [[ "$http_status" != "200" ]]; then
  echo "blog create failed"
  exit 4
fi
BLOG_ID=$(echo "$body" | jq -r '.id')

# 6) List blogs (check pagination metadata)
echo "\n== blog: list =="
call_and_parse GET "$BASE_URL/communication/blog"
echo "status=$http_status body=$body"
if [[ "$http_status" != "200" ]]; then
  echo "blog list failed"
  exit 5
fi
HAS_MORE=$(echo "$body" | jq -r '.has_more')
TOTAL_PAGES=$(echo "$body" | jq -r '.total_pages')

# 7) Update blog
echo "\n== blog: update =="
update_payload=$(jq -n --arg t "Hello updated $TS" '{title:$t}')
call_and_parse PUT "$BASE_URL/communication/blog/$BLOG_ID" "$update_payload"
echo "status=$http_status body=$body"

# 8) Delete blog
echo "\n== blog: delete =="
call_and_parse DELETE "$BASE_URL/communication/blog/$BLOG_ID"
echo "status=$http_status body=$body"
if [[ "$http_status" != "204" && "$http_status" != "200" ]]; then
  echo "blog delete failed"
  exit 6
fi

# 9) Create story
echo "\n== stories: create =="
story_payload=$(jq -n --arg m "https://example.com/$TS.png" --arg c "caption $TS" '{media_url:$m,caption:$c}')
call_and_parse POST "$BASE_URL/communication/stories" "$story_payload"
echo "status=$http_status body=$body"
if [[ "$http_status" != "200" ]]; then
  echo "story create failed"
  exit 7
fi
STORY_ID=$(echo "$body" | jq -r '.id')

# 10) List stories and check pagination metadata
echo "\n== stories: list =="
call_and_parse GET "$BASE_URL/communication/stories"
echo "status=$http_status body=$body"
if [[ "$http_status" != "200" ]]; then
  echo "stories list failed"
  exit 8
fi
HAS_MORE_STORY=$(echo "$body" | jq -r '.has_more')
TOTAL_PAGES_STORY=$(echo "$body" | jq -r '.total_pages')

# 11) Update story
echo "\n== stories: update =="
update_story_payload=$(jq -n --arg cap "bye $TS" '{caption:$cap}')
call_and_parse PUT "$BASE_URL/communication/stories/$STORY_ID" "$update_story_payload"
echo "status=$http_status body=$body"

# 12) Delete story
echo "\n== stories: delete =="
call_and_parse DELETE "$BASE_URL/communication/stories/$STORY_ID"
echo "status=$http_status body=$body"
if [[ "$http_status" != "204" && "$http_status" != "200" ]]; then
  echo "story delete failed"
  exit 9
fi

echo "\nAll endpoint checks completed successfully."
exit 0
