#!/usr/bin/env bash

cat res |
  rg '^- pkgs.([^ ]+) \(defined by ([^\)]+), (p?name): ([^,]+),' -or '$1 $2 $4' |
  sort -u -k2,3 |
  while read -r attr file value; do
    if ! result=$(ast-grep scan --inline-rules "$(sed "s/REPLACEME/$value/g" replace.yml)" ~/src/nixpkgs/"$file" -U 2>&1); then
      echo -e "\e[31mError while applying $attr in $file: $result\e[0m"
    fi
    if [[ -n $result ]]; then
      echo "$result for $attr in $file"
    else
      echo -e "\e[31mCould not apply for $attr in $file\e[0m"
    fi
  done |
  tee output
