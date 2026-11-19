#!/bin/sh
# shellcheck disable=SC2016 # The workflow's lines name their own $TAG.
# The workflow releases as record 0010 (design 3, 4, 5) and rule core.8
# have it, read from .github/workflows/release.yml's text, job by job:
#   landing   reads the head's Sluice-Tested trailer by
#             git interpret-trailers --parse;
#   version   runs only where that trailer was read, reckons the next
#             version from the newest tag, which is off main's history
#             (git-cliff --unreleased --bumped-version), and hands on no
#             tag that already exists;
#   binaries  writes the release's version (set-version.sh), builds by
#             cargo for its target, given nothing, and fails unless the
#             binary says that version; installs and runs no Nix, so that
#             run is on a runner with nothing in /nix/store, and builds
#             Linux's against musl alone;
#   release   writes the version, commits the files set-version.sh lists
#             by path as `chore(release): $TAG`, tags that commit, pushes
#             the tag alone, with no branch and no force, and publishes the
#             GitHub release at the tag it pushed;
# and no line of the workflow names BOOM_VERSION or --impure, so no build
# is given its version, and none pushes anything but the tag. It reads
# YAML, never a Markdown file (core.5); yamlfmt, under root:format, lays
# the workflow out, and this holds its steps. Each failure is one line,
# release: <job>: <what>, and every step missing is named.
#
# sluice's check-release.sh at boomdrivedev/sluice b6c30e78, for boom: its
# workflow is release.yml and its binary boom; the landing and version jobs
# are held to a landing's trailer, a missing step no longer hides the
# rest, the conventional job's checks are dropped, since no tier of
# boom's gate runs on GitHub, and no push but the tag's is held of the
# whole workflow.
set -u
root=$(cd "$(dirname "$0")" && pwd)
cd "$root" || exit 1
workflow=.github/workflows/release.yml
status=0

fail() {
  echo "release: $1: $2" >&2
  status=1
}

# The job $1's lines, from `  <job>:` to the next job at that indent, each
# shell line continued by `\` joined to the next, its leading space and a
# step's `- run: ` taken off.
job() {
  awk -v job="  $1:" '
    $0 == job { on = 1; next }
    on && /^  [a-z][a-z-]*:$/ { exit }
    on {
      line = $0
      sub(/^[ \t]+/, "", line)
      sub(/^- run: /, "", line)
      if (held != "") line = held " " line
      if (line ~ /\\$/) {
        sub(/[ \t]*\\$/, "", line)
        held = line
        next
      }
      held = ""
      print line
    }
  ' "$workflow"
}

# The number of the first of the lines $1 that is $2 exactly, or holds
# the fixed text $2 where $3 is `in`; empty where none is.
at() {
  if [ "${3:-}" = in ]; then
    printf '%s\n' "$1" | grep -nF -- "$2" | head -n 1 | cut -d : -f 1
  else
    printf '%s\n' "$1" | grep -nxF -- "$2" | head -n 1 | cut -d : -f 1
  fi
}

# Whether the steps $2 come in order in the job $1, each a line number,
# naming each step's words in $3 to $n; says each that is missing or out
# of order.
ordered() {
  name=$1
  shift
  previous=0
  said=""
  for step in "$@"; do
    number=${step%%:*}
    words=${step#*:}
    if [ -z "$number" ]; then
      fail "$name" "runs no $words"
      continue
    fi
    if [ "$number" -le "$previous" ]; then
      fail "$name" "runs $words ahead of $said, which it must follow"
    fi
    previous=$number
    said=$words
  done
}

landing=$(job landing)
version=$(job version)
binaries=$(job binaries)
release=$(job release)
for name in landing version binaries release; do
  [ -n "$(job "$name")" ] || fail "$name" "the workflow has no such job"
done

# landing: the head's Sluice-Tested trailer, as git reads trailers.
[ -n "$(at "$landing" 'git interpret-trailers --parse' in)" ] ||
  fail landing "runs no git interpret-trailers --parse"
[ -n "$(at "$landing" 'Sluice-Tested' in)" ] ||
  fail landing "reads no Sluice-Tested trailer"

# version: only on a landing, reckoned from the newest tag by version,
# handing on no tag that exists.
[ -n "$(at "$version" 'needs: landing')" ] ||
  fail version "does not need landing"
[ -n "$(at "$version" "if: needs.landing.outputs.tested != ''")" ] ||
  fail version "runs where no Sluice-Tested trailer was read: its condition is needs.landing.outputs.tested != ''"
[ -n "$(at "$version" 'git-cliff --unreleased --bumped-version' in)" ] ||
  fail version "runs no git-cliff --unreleased --bumped-version"
[ -n "$(at "$version" 'git rev-parse -q --verify "refs/tags/$tag"' in)" ] ||
  fail version "does not hold back a tag that already exists"

# binaries: the release's tree, built with nothing given, saying its
# version on a runner with nothing of Nix's, Linux's static.
ordered binaries \
  "$(at "$binaries" 'sh set-version.sh "${TAG#v}"'):sh set-version.sh \"\${TAG#v}\"" \
  "$(at "$binaries" 'cargo build --release --locked -p boom --target "$TARGET"'):cargo build --release --locked -p boom --target \"\$TARGET\"" \
  "$(at "$binaries" 'test "$("target/$TARGET/release/boom" --version)" = "boom ${TAG#v}"'):test that target/\$TARGET/release/boom --version says boom \${TAG#v}"
if printf '%s\n' "$binaries" | grep -qE 'install-nix-action|(^|[ ;&|(])nix( |$)'; then
  fail binaries "installs or runs Nix: the binaries it builds must run, and run there, on a runner with nothing in /nix/store"
fi
targets=$(printf '%s\n' "$binaries" | sed -n 's/^target: *//p')
[ -n "$targets" ] || fail binaries "names no target in its matrix"
for target in $targets; do
  case $target in
  *-linux-musl) ;;
  *-linux-*) fail binaries "builds $target: Linux's binaries are static, against musl" ;;
  esac
done

# release: the release commit, its tag pushed alone, the release at the tag.
commit=$(printf '%s\n' "$release" | grep -nE 'commit .*-m "chore\(release\): \$TAG" .*-- "\$\{files\[@\]\}"$' | head -n 1 | cut -d : -f 1)
ordered release \
  "$(at "$release" 'sh set-version.sh "${TAG#v}"'):sh set-version.sh \"\${TAG#v}\"" \
  "$(at "$release" 'sh set-version.sh --files' in):the files of sh set-version.sh --files" \
  "$commit:git commit -m \"chore(release): \$TAG\" -- the files set-version.sh lists" \
  "$(at "$release" 'git tag "$TAG"'):git tag \"\$TAG\"" \
  "$(at "$release" 'git push origin "refs/tags/$TAG"'):git push origin \"refs/tags/\$TAG\", the tag alone" \
  "$(at "$release" 'gh release create "$TAG" --verify-tag' in):gh release create \"\$TAG\" --verify-tag"
if printf '%s\n' "$release" | grep -E 'commit ' | grep -Eq '(^| )(-a|--all|-am)( |$)'; then
  fail release "commits with -a: it commits the files set-version.sh lists, by path"
fi
pushes=$(printf '%s\n' "$release" | grep -cE '(^|[ ;&|])git( -c [^ ]+)* push( |$)')
if [ "$pushes" -ne 1 ]; then
  fail release "pushes $pushes times: its one push is the tag, alone"
fi
printf '%s\n' "$release" | grep -E 'git( -c [^ ]+)* push( |$)' | while IFS= read -r push; do
  [ "$push" = 'git push origin "refs/tags/$TAG"' ] ||
    echo "release: release: pushes other than the tag alone, with no branch and no force: $push" >&2
done
if printf '%s\n' "$release" | grep -E 'git( -c [^ ]+)* push( |$)' | grep -vxqF 'git push origin "refs/tags/$TAG"'; then
  status=1
fi
if printf '%s\n' "$release" | grep -qF -- '--target'; then
  fail release "names a --target: the release is published at the tag it pushed"
fi

# The whole workflow: no build given its version, and no push but the
# tag's.
grep -nF 'BOOM_VERSION' "$workflow" | while IFS= read -r line; do
  echo "release: workflow: names BOOM_VERSION, at line ${line%%:*}" >&2
done
grep -nF -- '--impure' "$workflow" | while IFS= read -r line; do
  echo "release: workflow: names --impure, at line ${line%%:*}" >&2
done
if grep -qF -e 'BOOM_VERSION' -e '--impure' "$workflow"; then
  status=1
fi
for name in landing version binaries; do
  job "$name" | grep -E '(^|[ ;&|])git( -c [^ ]+)* push( |$)' | while IFS= read -r push; do
    echo "release: $name: pushes: its workflow's one push is the release job's tag: $push" >&2
  done
  if job "$name" | grep -qE '(^|[ ;&|])git( -c [^ ]+)* push( |$)'; then
    status=1
  fi
done

[ "$status" -ne 0 ] || echo "release: the workflow releases as record 0010 has it"
exit $status
