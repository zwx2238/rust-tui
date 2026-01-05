FROM python:3.11-slim

ARG APT_MIRROR=mirrors.tuna.tsinghua.edu.cn

RUN set -e; \
    if [ -f /etc/apt/sources.list ]; then \
        cp /etc/apt/sources.list /tmp/sources.list.bak; \
        sed -i "s|https\\?://deb.debian.org|https://${APT_MIRROR}|g" /etc/apt/sources.list; \
        sed -i "s|https\\?://security.debian.org|https://${APT_MIRROR}|g" /etc/apt/sources.list; \
    fi; \
    if [ -f /etc/apt/sources.list.d/debian.sources ]; then \
        cp /etc/apt/sources.list.d/debian.sources /tmp/debian.sources.bak; \
        sed -i "s|https\\?://deb.debian.org|https://${APT_MIRROR}|g" /etc/apt/sources.list.d/debian.sources; \
        sed -i "s|https\\?://security.debian.org|https://${APT_MIRROR}|g" /etc/apt/sources.list.d/debian.sources; \
    fi; \
    if ! apt-get update; then \
        if [ -f /tmp/sources.list.bak ]; then cp /tmp/sources.list.bak /etc/apt/sources.list; fi; \
        if [ -f /tmp/debian.sources.bak ]; then cp /tmp/debian.sources.bak /etc/apt/sources.list.d/debian.sources; fi; \
        apt-get update; \
    fi; \
    if ! apt-get install -y --no-install-recommends git patch bash ca-certificates; then \
        if [ -f /tmp/sources.list.bak ]; then cp /tmp/sources.list.bak /etc/apt/sources.list; fi; \
        if [ -f /tmp/debian.sources.bak ]; then cp /tmp/debian.sources.bak /etc/apt/sources.list.d/debian.sources; fi; \
        apt-get update; \
        apt-get install -y --no-install-recommends git patch bash ca-certificates; \
    fi; \
    rm -rf /var/lib/apt/lists/*
