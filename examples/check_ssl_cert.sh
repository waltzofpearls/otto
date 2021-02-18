#!/bin/sh

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <host>" >&2
  exit 1
fi

host_n_port="$1:443"

if true | openssl s_client -connect "$host_n_port" 2>/dev/null | \
  openssl x509 -noout -checkend 0; then
  >&2 echo "SSL cert for $1 is not expired"
  >&2 echo "SSL cert validity:" | openssl s_client -connect "$host_n_port" 2>/dev/null | \
    openssl x509 -noout -dates
  exit 0
else
  >&2 echo "SSL cert for $1 is expired"
  >&2 echo "SSL cert validity:" | openssl s_client -connect "$host_n_port" 2>/dev/null | \
    openssl x509 -noout -dates
  exit 1
fi
