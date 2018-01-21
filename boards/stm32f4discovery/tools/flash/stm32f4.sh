#!/usr/bin/env bash

if [[ -z "$APP_BASE_ADDR" ]]; then
  echo "APP_BASE_ADDR not set"
  exit 1
fi

TMPCONF=$(mktemp)
TMPAPP=$(mktemp)

for app in "$@"
do
  cat "$app" >> $TMPAPP
done

wc $TMPAPP

printf "\0\0\0\0" >> $TMPAPP

cat << EOF > $TMPCONF
source [find interface/stlink-v2.cfg]
transport select hla_swd
source [find target/stm32f4x.cfg]
reset_config srst_only
EOF

openocd -f $TMPCONF -c "init; reset halt; flash write_image erase $TMPAPP $APP_BASE_ADDR bin; reset; shutdown"

rm $TMPCONF
rm $TMPAPP
