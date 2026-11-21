#!/usr/bin/env bash
# The plugin's PreToolUse hook, run as Claude Code runs it: its command,
# read from hooks/hooks.json, by `sh -c`, with Claude Code's event on its
# standard input (term-check, claude-code-plugin). A `boom` of the
# script's own proves the hook calls boom by its command line from PATH
# and hands it the event; the boom the tree builds proves the hook says
# nothing and makes no store where boom finds no repository of the file,
# and lets a tool that writes no file's text pass. That an old word stops
# a write is proven in crates/boom/tests/terms.rs, with a store whose term
# a person accepted with the tests' keys, which a shell cannot make
# (record 0006). Each case that fails prints `claude-code: <case>: <what>`,
# and every case runs.

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

# The hook's matcher and command, as Claude Code reads them: the one
# command hook of PreToolUse.
hooks=$(jq -c '[.hooks.PreToolUse[]? | {matcher, commands: [.hooks[]? | select(.type == "command") | .command]}]' "$plugin/hooks/hooks.json")
if [ "$(printf '%s' "$hooks" | jq 'length')" != 1 ] ||
  [ "$(printf '%s' "$hooks" | jq '.[0].commands | length')" != 1 ]; then
  echo "claude-code: hooks: PreToolUse holds not one command hook: $hooks" >&2
  exit 1
fi
matcher=$(printf '%s' "$hooks" | jq -r '.[0].matcher')
command=$(printf '%s' "$hooks" | jq -r '.[0].commands[0]')

# Under sluice's sandbox only SLUICE_TMP is writable beside the checkout.
scratch=$(mktemp -d "${SLUICE_TMP:-${TMPDIR:-/tmp}}/boom-claude-write.XXXXXX")
trap 'rm -rf "$scratch"' EXIT
scratch=$(cd "$scratch" && pwd -P)

built=$scratch/built
mkdir -p "$built"
ln -s "$target/debug/boom" "$built/boom"

# A boom of the script's own, recording how it was called and what it
# read.
fake=$scratch/fake
record=$scratch/record
mkdir -p "$fake" "$record"
cat >"$fake/boom" <<SCRIPT
#!/bin/sh
printf '%s\n' "\$*" >"$record/args"
cat >"$record/input"
SCRIPT
chmod +x "$fake/boom"

failures=0
fail() { # case what
  echo "claude-code: $1: $2" >&2
  failures=$((failures + 1))
}

# A repository's folder whose configuration names that remote as its
# origin.
repository() { # name remote
  mkdir -p "$scratch/$1/.git"
  printf '[remote "origin"]\n\turl = %s\n' "$2" >"$scratch/$1/.git/config"
  printf '%s\n' "$scratch/$1"
}

# Runs the hook's command on an event with a folder first on PATH, its
# store's home a scratch folder of its own: sets code, out, err and data.
hook() { # name folder bin event
  data=$scratch/data-$1
  mkdir -p "$data" "$scratch/home-$1"
  (
    cd "$2" &&
      printf '%s' "$4" | PATH="$3:$PATH" XDG_DATA_HOME="$data" HOME="$scratch/home-$1" \
        BOOM_REPOSITORY="" sh -c "$command" >"$scratch/out-$1" 2>"$scratch/err-$1"
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

scenario="The hook matches the tools that write a file's text"
[ "$matcher" = "Write|Edit|MultiEdit" ] || fail "$scenario" "its matcher is $matcher"

shop=$(repository shop git@github.com:example/shop.git)
event=$(jq -cn --arg file "$shop/README.md" \
  '{hook_event_name: "PreToolUse", tool_name: "Write", tool_input: {file_path: $file, content: "The keeper signs.\n"}}')

scenario="The hook runs boom"
hook runs "$shop" "$fake" "$event"
[ "$code" = 0 ] || fail "$scenario" "exited $code, not 0: $err"
args=$(cat "$record/args" 2>/dev/null)
[ "$args" = "terms check --hook --if-declared" ] ||
  fail "$scenario" "boom was called as '${args:-nothing}', not 'terms check --hook --if-declared'"
input=$(cat "$record/input" 2>/dev/null)
[ "$input" = "$event" ] || fail "$scenario" "boom read '${input:-nothing}', not the event"

scenario="A file in no registered repository"
hook unregistered "$shop" "$built" "$event"
silent "$scenario"

scenario="Another tool"
bash_event=$(jq -cn '{hook_event_name: "PreToolUse", tool_name: "Bash", tool_input: {command: "echo keeper"}}')
hook bash "$shop" "$built" "$bash_event"
silent "$scenario"

[ "$failures" = 0 ] || exit 1
echo "claude-code: the write hook passes every case"
