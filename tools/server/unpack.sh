#!/bin/bash
# Распаковать /var/tmp/blackwire-site.tgz в корень сайта game.globthe.top (web2). Зовёт update-site.ps1.
# Прежний index.html сохраняется в /root рядом с датой.
set -e
W=/var/www/clients/client0/web2/web
[ -f "$W/index.html" ] && cp -p "$W/index.html" "/root/game-site-index.$(date +%Y%m%d%H%M).bak"
tar -xzf /var/tmp/blackwire-site.tgz -C "$W" --no-same-owner --no-same-permissions --no-overwrite-dir
chown -R web2:client0 "$W"
find "$W" -mindepth 1 -type d -exec chmod 755 {} +
find "$W" -type f -exec chmod 644 {} +
rm -f /var/tmp/blackwire-site.tgz /var/tmp/blackwire-site-unpack.sh
echo "залито: $(du -sh "$W" | cut -f1)"
