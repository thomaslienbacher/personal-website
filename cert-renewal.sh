#!/bin/sh

echo "Trying to refresh on $(date)"
systemctl start httpd
certbot renew --webroot-path /var/www/html/
systemctl stop httpd
nginx -s reload

