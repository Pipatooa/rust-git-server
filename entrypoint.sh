#!/usr/bin/env sh

grep HostKey /etc/ssh/sshd_config | sed -e 's/HostKey //' |
while read -r file; do
  if [ ! -f "$file" ]
  then
    mkdir -p /tmp/hostkey-gen/etc/ssh
    ssh-keygen -A -f /tmp/hostkey-gen
    cp -nv /tmp/hostkey-gen/etc/ssh/ssh_host_*_key /etc/ssh/keys
    break
  fi
done

exec "$@"
