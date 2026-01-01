#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "用法: $0 <query> [count] [market]" >&2
  echo "需要填写脚本内的 BING_ENDPOINT/BING_API_KEY" >&2
  exit 1
fi

QUERY="$1"
COUNT="${2:-5}"
MARKET="${3:-zh-CN}"

# TODO: 填入你的 Bing Web Search endpoint 与 key
BING_ENDPOINT="https://api.bing.microsoft.com/v7.0/search"
BING_API_KEY=""

if [[ -z "$BING_API_KEY" ]]; then
  echo "请先在脚本里填写 BING_API_KEY" >&2
  exit 2
fi

curl -sS "$BING_ENDPOINT" \
  -G \
  --data-urlencode "q=$QUERY" \
  --data-urlencode "count=$COUNT" \
  --data-urlencode "mkt=$MARKET" \
  -H "Ocp-Apim-Subscription-Key: $BING_API_KEY" \
  | python3 - <<'PY'
import json, sys
try:
    data = json.load(sys.stdin)
except Exception as e:
    print("解析失败:", e, file=sys.stderr)
    sys.exit(3)
items = data.get("webPages", {}).get("value", [])
if not items:
    print("结果为空")
    sys.exit(0)
for i, item in enumerate(items, 1):
    print(f"[{i}] {item.get('name','')}")
    print(f"    {item.get('url','')}")
    snippet = item.get('snippet','').strip()
    if snippet:
        print(f"    {snippet}")
PY
