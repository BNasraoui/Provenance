#!/usr/bin/env bash
# Emits one line per completed CI check on PR 118, then exits when all are done.
prev=""
for i in $(seq 1 60); do
  s=$(gh pr checks 118 --json name,bucket 2>/dev/null)
  if [ -z "$s" ]; then
    sleep 30
    continue
  fi
  cur=$(printf '%s' "$s" | jq -r '.[] | select(.bucket != "pending") | "\(.name): \(.bucket)"' | sort)
  printf '%s\n' "$cur" | grep -Fxv -f <(printf '%s\n' "$prev") 2>/dev/null | grep -v '^$'
  prev="$cur"
  if printf '%s' "$s" | jq -e 'length > 0 and all(.[]; .bucket != "pending")' >/dev/null 2>&1; then
    echo "ALL CHECKS COMPLETE"
    break
  fi
  sleep 30
done
