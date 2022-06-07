#!/bin/sh

systemctl start httpd
certbot renew --webroot-path /var/www/html/
systemctl stop httpd

