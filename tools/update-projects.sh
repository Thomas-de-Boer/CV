#!/usr/bin/env bash
# Writes assets/projects.json: the repositories pinned on the GitHub profile,
# in the order they are pinned. A repository's picture is the one uploaded on
# GitHub under Settings -> Social preview (null if there is none). Its website
# is the "Website" field in the repository's About box (null if it is empty or
# not an http(s) URL). Needs the GitHub CLI (`gh`), logged in or with
# GH_TOKEN set. Run from the repository root: bash tools/update-projects.sh
set -euo pipefail

USER_LOGIN="${1:-Thomas-de-Boer}"
OUT="assets/projects.json"

gh api graphql -f login="$USER_LOGIN" -f query='
  query($login: String!) {
    user(login: $login) {
      pinnedItems(first: 6, types: REPOSITORY) {
        nodes {
          ... on Repository {
            name
            description
            url
            homepageUrl
            isPrivate
            primaryLanguage { name }
            usesCustomOpenGraphImage
            openGraphImageUrl
            repositoryTopics(first: 4) { nodes { topic { name } } }
          }
        }
      }
    }
  }' --jq '[.data.user.pinnedItems.nodes[] | select(.isPrivate | not) | {
    name,
    description,
    language: (.primaryLanguage.name // null),
    topics: [.repositoryTopics.nodes[].topic.name],
    image: (if .usesCustomOpenGraphImage then .openGraphImageUrl else null end),
    website: ((.homepageUrl // "") | gsub("^\\s+|\\s+$"; "") | if test("^https?://"; "i") then . else null end),
    url
  }]' > "$OUT.tmp"

# Keep the old file if GitHub returned nothing usable.
if [ "$(tr -d '[:space:]' < "$OUT.tmp")" = "[]" ] || [ ! -s "$OUT.tmp" ]; then
  rm -f "$OUT.tmp"
  echo "No pinned repositories found for $USER_LOGIN; $OUT left unchanged." >&2
  exit 1
fi
mv "$OUT.tmp" "$OUT"
echo "Wrote $OUT"
