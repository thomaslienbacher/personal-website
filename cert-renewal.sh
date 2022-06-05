#!/bin/sh

systemctl start httpd
certbot renew   
systemctl stop httpd

