#!/usr/bin/env python3
import argparse
import os
import urllib.parse
import urllib.request


def bark_push(key: str, title: str, body: str) -> None:
    title_q = urllib.parse.quote(title, safe="")
    body_q = urllib.parse.quote(body, safe="")
    url = f"https://api.day.app/{key}/{title_q}/{body_q}"
    req = urllib.request.Request(url, method="GET")
    with urllib.request.urlopen(req, timeout=5) as resp:
        resp.read()


def resolve_event(args: argparse.Namespace) -> str:
    if args.event:
        return args.event
    return os.getenv("HOOK_EVENT", "unknown")


def resolve_title(args: argparse.Namespace) -> str:
    if args.title:
        return args.title
    return os.getenv("BARK_TITLE", "DeepChat")


def resolve_body(args: argparse.Namespace, event: str) -> str:
    if args.body:
        return args.body
    if args.message:
        return args.message
    return f"event: {event}"


def load_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Send Bark notification")
    parser.add_argument("--event", default="", help="hook event name")
    parser.add_argument("--title", default="", help="notification title")
    parser.add_argument("--body", default="", help="notification body")
    parser.add_argument("message", nargs="?", default="")
    return parser.parse_args()


def resolve_key() -> str:
    return os.getenv("DEEPCHAT_BARK_KEY") or os.getenv("BARK_KEY", "")


def main() -> int:
    args = load_args()
    key = resolve_key()
    if not key:
        print("missing BARK key: set DEEPCHAT_BARK_KEY or BARK_KEY", flush=True)
        return 1
    event = resolve_event(args)
    title = resolve_title(args)
    body = resolve_body(args, event)[:200]
    bark_push(key, title, body)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
