#!/bin/sh

pkill personal-websit 
cd /root/personal-website
./target/release/personal-website & disown   
echo "Restart at $(date)" 

