#!/bin/bash

ACCOUNT=${1:healthforge}

# Grab session
token=$(cat ~/.1pw_token ||:)
echo Token: "$token"

# Get a list of all items from op
if ! allItems=$(op list items --session "$token"); then
  echo "Login required"
  if token=$(op signin "$ACCOUNT" --output=raw); then
    echo "$token" > ~/.1pw_token
    exit 0
  else
    exit 1
  fi
fi

#echo "$allItems" > itemCache
#allItems=$(cat itemCache)

# format them in a searchable way
list=$(echo "$allItems" | jq -r ".[] | .overview.title")

if ! choice=$(echo "$list" | dmenu -b -l 20 -p "Search for password: "); then
  exit 2
fi

# Extract all fields
fields=$(op get item "$choice" --session "$token" | jq -r '.details.fields[] | [.name, .value] | @csv')
totp=$(op get totp "$choice" --session "$token")
if ! pw=$(printf '%s\n"totp","%s"' "$fields" "$totp" | dmenu -b -l 20); then
  exit 2
fi

value=$(echo "$pw" | cut -d, -f2 | sed 's/^"\(.*\)"/\1/')

echo "Copying value $value into paste buffer"
echo $value | xsel -b
