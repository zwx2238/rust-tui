#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "用法: $0 <query> [max_results] [search_depth] [include_domains] [exclude_domains]" >&2
  echo "search_depth: basic 或 advanced" >&2
  echo "include_domains/exclude_domains: 逗号分隔域名，如 github.com,gofrp.org" >&2
  exit 1
fi

QUERY="$1"
MAX_RESULTS="${2:-5}"
SEARCH_DEPTH="${3:-basic}"
INCLUDE_DOMAINS="${4:-${TAVILY_INCLUDE_DOMAINS:-}}"
EXCLUDE_DOMAINS="${5:-${TAVILY_EXCLUDE_DOMAINS:-}}"

TAVILY_API_KEY="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"

TMP_BODY="$(mktemp)"
TMP_HEADERS="$(mktemp)"
trap 'rm -f "$TMP_BODY" "$TMP_HEADERS"' EXIT

PAYLOAD="$(
  QUERY="$QUERY" MAX_RESULTS="$MAX_RESULTS" SEARCH_DEPTH="$SEARCH_DEPTH" \
  INCLUDE_DOMAINS="$INCLUDE_DOMAINS" EXCLUDE_DOMAINS="$EXCLUDE_DOMAINS" \
  TAVILY_API_KEY="$TAVILY_API_KEY" \
  python3 - <<'PY'
import json, os

payload = {
    "api_key": os.environ["TAVILY_API_KEY"],
    "query": os.environ["QUERY"],
    "max_results": int(os.environ["MAX_RESULTS"]),
    "search_depth": os.environ["SEARCH_DEPTH"],
}

include_domains = os.environ.get("INCLUDE_DOMAINS", "").strip()
exclude_domains = os.environ.get("EXCLUDE_DOMAINS", "").strip()

if include_domains:
    payload["include_domains"] = [d for d in include_domains.split(",") if d]
if exclude_domains:
    payload["exclude_domains"] = [d for d in exclude_domains.split(",") if d]

print(json.dumps(payload, ensure_ascii=False))
PY
)"

curl -sS -D "$TMP_HEADERS" -o "$TMP_BODY" \
  "https://api.tavily.com/search" \
  -H "Content-Type: application/json" \
  -d "$PAYLOAD"

STATUS=$(awk 'NR==1{print $2}' "$TMP_HEADERS")
if [[ -z "$STATUS" ]]; then
  echo "请求失败：没有返回 HTTP 状态（可能是代理/网络问题）" >&2
  sed -n '1,5p' "$TMP_HEADERS" >&2 || true
  exit 3
fi

if [[ "$STATUS" -lt 200 || "$STATUS" -ge 300 ]]; then
  echo "请求失败：HTTP $STATUS" >&2
  sed -n '1,5p' "$TMP_HEADERS" >&2 || true
  echo "响应体（前200字）:" >&2
  head -c 200 "$TMP_BODY" >&2 || true
  echo >&2
  exit 4
fi

python3 - <<'PY' "$TMP_BODY"
import json, sys
path = sys.argv[1]
try:
    with open(path, 'r', encoding='utf-8') as f:
        data = json.load(f)
except Exception as e:
    print("解析失败:", e, file=sys.stderr)
    print("原始响应（前200字）:", file=sys.stderr)
    try:
        with open(path, 'r', encoding='utf-8') as f:
            print(f.read(200), file=sys.stderr)
    except Exception:
        pass
    sys.exit(5)

results = data.get("results", [])
if not results:
    print("结果为空")
    sys.exit(0)
for i, item in enumerate(results, 1):
    print(f"[{i}] {item.get('title','')}")
    print(f"    {item.get('url','')}")
    snippet = item.get('content','').strip()
    if snippet:
        print(f"    {snippet}")
PY
