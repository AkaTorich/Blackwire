#!/bin/bash
# Поставить или обновить онлайн-сервер Blackwire на Linux (запускать от root
# в папке, где лежат blackwire-server, blackwire-server.service и этот файл).
#   bash install.sh [домен] [порт]      по умолчанию: game.globthe.top 7777
# Мир и игроки лежат в /opt/blackwire-server/data и при обновлении не трогаются.
set -e
DOMAIN="${1:-game.globthe.top}"
PORT="${2:-7777}"
D=/opt/blackwire-server
cd "$(dirname "$0")"

id blackwire >/dev/null 2>&1 || useradd --system --home-dir "$D" --shell /usr/sbin/nologin blackwire
mkdir -p "$D/data" "$D/tls"

# сертификат сайта: Let's Encrypt (certbot) или папка ssl сайта ISPConfig
CERT=""; KEY=""
if [ -f "/etc/letsencrypt/live/$DOMAIN/fullchain.pem" ]; then
  CERT="/etc/letsencrypt/live/$DOMAIN/fullchain.pem"; KEY="/etc/letsencrypt/live/$DOMAIN/privkey.pem"
else
  for s in /var/www/clients/*/web*/ssl /var/www/"$DOMAIN"/ssl; do
    if [ -f "$s/$DOMAIN-le.crt" ] && [ -f "$s/$DOMAIN-le.key" ]; then
      CERT="$s/$DOMAIN-le.crt"; KEY="$s/$DOMAIN-le.key"
      [ -f "$s/$DOMAIN-le.bundle" ] && BUNDLE="$s/$DOMAIN-le.bundle"
      break
    fi
  done
fi
if [ -z "$CERT" ]; then
  echo "сертификат для $DOMAIN не найден — сервер не запущен"; exit 1
fi
echo "сертификат: $CERT"

# копия сертификата для службы (ключ сайта читает только root); обновляется раз в сутки
cat > /usr/local/sbin/blackwire-cert-sync <<SYNC
#!/bin/bash
# Скопировать сертификат $DOMAIN для онлайн-сервера Blackwire (сервер перечитает его сам).
set -e
T=\$(mktemp)
cat "$CERT" > "\$T"
# в файле только сертификат сайта — дописать цепочку
if [ -f "${BUNDLE:-/nonexistent}" ] && [ "\$(grep -c 'BEGIN CERTIFICATE' "\$T")" = 1 ]; then cat "${BUNDLE:-/nonexistent}" >> "\$T"; fi
if ! cmp -s "\$T" "$D/tls/fullchain.pem"; then
  install -m 640 -o root -g blackwire "\$T" "$D/tls/fullchain.pem"
  install -m 640 -o root -g blackwire "$KEY" "$D/tls/privkey.pem"
fi
rm -f "\$T"
SYNC
chmod 755 /usr/local/sbin/blackwire-cert-sync
/usr/local/sbin/blackwire-cert-sync
chown root:blackwire "$D/tls"; chmod 750 "$D/tls"
echo "17 4 * * * root /usr/local/sbin/blackwire-cert-sync" > /etc/cron.d/blackwire-cert-sync

# бинарник и служба
systemctl stop blackwire-server 2>/dev/null || true
install -m 755 -o root -g root blackwire-server "$D/blackwire-server"
chown -R blackwire:blackwire "$D/data"
sed "s/--port 7777/--port $PORT/" blackwire-server.service > /etc/systemd/system/blackwire-server.service
systemctl daemon-reload
systemctl enable --now blackwire-server
sleep 2
systemctl --no-pager --lines=8 status blackwire-server || true

# порт наружу, если включён ufw
if command -v ufw >/dev/null && ufw status | grep -q "Status: active"; then ufw allow "$PORT/tcp"; fi
echo "готово: $DOMAIN:$PORT"
