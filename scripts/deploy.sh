rsync -av ./release/* artcx:/app/art.cx/

ssh artcx <<EOF
cd /app/art.cx
sudo setcap 'cap_net_bind_service=+ep' server
sudo systemctl restart artcx
EOF

