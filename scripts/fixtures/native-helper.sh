#!/bin/sh
# Protocol-only native helper: answers hello with ready and every event with done.
read -r line
printf '%s\n' '{"type":"ready"}'
while read -r line; do
  printf '%s\n' '{"type":"done","id":1,"code":0}'
done
