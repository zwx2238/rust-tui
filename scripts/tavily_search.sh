#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "用法: $0 <query> [max_results] [search_depth]" >&2
  echo "search_depth: basic 或 advanced" >&2
  exit 1
fi

QUERY="$1"
MAX_RESULTS="${2:-5}"
SEARCH_DEPTH="${3:-basic}"

TAVILY_API_KEY="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"

TMP_BODY="$(mktemp)"
TMP_HEADERS="$(mktemp)"
trap 'rm -f "$TMP_BODY" "$TMP_HEADERS"' EXIT

curl -sS -D "$TMP_HEADERS" -o "$TMP_BODY" \
  "https://api.tavily.com/search" \
  -H "Content-Type: application/json" \
  -d "{\"api_key\":\"$TAVILY_API_KEY\",\"query\":\"$QUERY\",\"max_results\":$MAX_RESULTS,\"search_depth\":\"$SEARCH_DEPTH\"}"

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
