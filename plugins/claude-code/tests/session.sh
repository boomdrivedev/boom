#!/usr/bin/env bash
# The plugin's SessionStart hook, run as Claude Code runs it: its command,
# read from hooks/hooks.json, by `sh -c` in a session's folder, its
# standard output what the agent's context is given (design 9). A `boom`
# of the script's own proves the hook calls boom by its command line from
# PATH, in the session's folder, and hands its output on as it is; the
# boom the tree builds proves the hook says nothing and makes no store
# where no repository is declared, and shows a declaration boom cannot
# read. The claims given for a declared repository are boom query's,
# proven in crates/boom/tests/query.rs with the tests' keys, which a shell
# cannot make (record 0006). Each case that fails prints
# `claude-code: <case>: <what>`, and every case runs.

set -u
here=$(cd "$(dirname "$0")" && pwd)
plugin=$(cd "$here/.." && pwd)
root=$(cd "$plugin/../.." && pwd)

# The boom the tree builds, by the Rust the tree pins.
if ! (cd "$root" && cargo build --locked -p boom); then
  echo "claude-code: build: cargo build --locked -p boom failed" >&2
  exit 1
fi
target=$(cd "$root" && cargo metadata --format-version 1 --no-deps --locked | jq -r .target_directory)
if [ ! -x "$target/debug/boom" ]; then
  echo "claude-code: build: no boom at $target/debug/boom" >&2
  exit 1
fi

# The hook's command, as Claude Code reads it: the one command hook of
# SessionStart.
commands=$(jq -r '[.hooks.SessionStart[]?.hooks[]? | select(.type == "command") | .command]' "$plugin/hooks/hooks.json")
if [ "$(printf '%s' "$commands" | jq length)" != 1 ]; then
  echo "claude-code: hooks: SessionStart holds not one command hook: $commands" >&2
  exit 1
fi
command=$(printf '%s' "$commands" | jq -r '.[0]')

# Under sluice's sandbox only SLUICE_TMP is writable beside the checkout.
scratch=$(mktemp -d "${SLUICE_TMP:-${TMPDIR:-/tmp}}/boom-claude-session.XXXXXX")
trap 'rm -rf "$scratch"' EXIT
scratch=$(cd "$scratch" && pwd -P)

# The built boom, first on PATH, by a folder holding it alone.
built=$scratch/built
mkdir -p "$built"
ln -s "$target/debug/boom" "$built/boom"

# A boom of the script's own, recording how it was called and printing a
# line.
fake=$scratch/fake
record=$scratch/record
mkdir -p "$fake" "$record"
cat >"$fake/boom" <<EOF
#!/bin/sh
printf '%s\n' "\$*" >"$record/args"
pwd -P >"$record/cwd"
echo "1 claim stands for shop, at entry 3 of the store's log."
EOF
chmod +x "$fake/boom"

# A repository's folder: a .git folder and, where given, a declaration of
# those words.
repository() { # name [declaration]
  mkdir -p "$scratch/$1/.git"
  if [ $# -gt 1 ]; then
    mkdir -p "$scratch/$1/.config"
    printf '%s\n' "$2" >"$scratch/$1/.config/boom.toml"
  fi
  printf '%s\n' "$scratch/$1"
}

failures=0
fail() { # case what
  echo "claude-code: $1: $2" >&2
  failures=$((failures + 1))
}

# Runs the hook's command in a folder with a folder first on PATH, its
# store's home a scratch folder of its own: sets code, out, err and data.
hook() { # name folder bin
  data=$scratch/data-$1
  mkdir -p "$data" "$scratch/home-$1"
  (
    cd "$2" &&
      PATH="$3:$PATH" XDG_DATA_HOME="$data" HOME="$scratch/home-$1" \
        sh -c "$command" >"$scratch/out-$1" 2>"$scratch/err-$1"
  )
  code=$?
  out=$(cat "$scratch/out-$1")
  err=$(cat "$scratch/err-$1")
}

# Holds a run to exit 0, both streams empty and no store made.
silent() { # case
  [ "$code" = 0 ] || fail "$1" "exited $code, not 0: $err"
  [ -z "$out" ] || fail "$1" "standard output is not empty: $out"
  [ -z "$err" ] || fail "$1" "standard error is not empty: $err"
  made=$(find "$data" -mindepth 1 -print)
  [ -z "$made" ] || fail "$1" "a store was made under XDG_DATA_HOME: $made"
}

scenario="The hook runs boom"
declared=$(repository declared 'repository = "shop"')
hook runs "$declared" "$fake"
[ "$code" = 0 ] || fail "$scenario" "exited $code, not 0: $err"
args=$(cat "$record/args" 2>/dev/null)
[ "$args" = "query --if-declared" ] || fail "$scenario" "boom was called as '${args:-nothing}', not 'query --if-declared'"
cwd=$(cat "$record/cwd" 2>/dev/null)
[ "$cwd" = "$declared" ] || fail "$scenario" "boom ran in ${cwd:-no folder}, not $declared"
[ "$out" = "1 claim stands for shop, at entry 3 of the store's log." ] ||
  fail "$scenario" "the hook's output is not boom's line: $out"

scenario="A repository that declares nothing"
undeclared=$(repository undeclared)
hook undeclared "$undeclared" "$built"
silent "$scenario"

scenario="Outside a repository"
outside=$scratch/outside
mkdir -p "$outside/.config"
printf 'repository = "shop"\n' >"$outside/.config/boom.toml"
above=$outside
while [ "$above" != / ]; do
  if [ -e "$above/.git" ]; then
    fail "$scenario" "$outside is inside the repository at $above"
    break
  fi
  above=$(dirname "$above")
done
hook outside "$outside" "$built"
silent "$scenario"

scenario="A name not in kebab case"
named=$(repository named 'repository = "Shop"')
hook named "$named" "$built"
[ "$code" = 2 ] || fail "$scenario" "exited $code, not 2: $err"
case $err in
*.config/boom.toml*) ;;
*) fail "$scenario" "standard error does not name .config/boom.toml: $err" ;;
esac

[ "$failures" = 0 ] || exit 1
echo "claude-code: the session hook passes every case"
