#!/usr/bin/env bash
# A stop whose reason has more than 10,000 characters, given by the
# plugin's PreToolUse hook to the pinned Claude Code (session-context;
# design 3). Claude Code caps the text of a hook at 10,000 characters for
# additionalContext, systemMessage and plain standard output. Its docs do
# not say whether it caps permissionDecisionReason. This test shows what
# the agent gets.
#
# Claude Code runs in print mode with the plugin from this tree. Its
# Messages API is tests/api.mjs on 127.0.0.1, so no request leaves the
# host. The API asks for one Write. A `boom` of the script's own stops the
# Write with a reason of 15,000 characters. The test passes where the
# tool_result of the next request holds the whole reason, and the file is
# not written. The home is a scratch folder, and the variables of Claude
# Code and Anthropic are unset, as plugins/claude.sh does (core.3). Each
# case that fails prints `claude-code: cap: <what>`.

set -u
here=$(cd "$(dirname "$0")" && pwd)
plugin=$(cd "$here/.." && pwd)

# Under sluice's sandbox only SLUICE_TMP is writable beside the checkout.
scratch=$(mktemp -d "${SLUICE_TMP:-${TMPDIR:-/tmp}}/boom-claude-cap.XXXXXX")
scratch=$(cd "$scratch" && pwd -P)
api_pid=
trap '[ -n "$api_pid" ] && kill "$api_pid" 2>/dev/null; rm -rf "$scratch"' EXIT
mkdir -p "$scratch/home" "$scratch/work" "$scratch/log" "$scratch/bin"

fail() { # what
  echo "claude-code: cap: $1" >&2
  exit 1
}

# The reason: 15,000 characters, with a mark at its start, past 10,000
# and at its end.
filler() { # count
  printf "%$1s" '' | tr ' ' r
}
reason=$scratch/reason
{
  printf '<<START>>'
  filler 9991
  printf '<<PAST10000>>'
  filler 4980
  printf '<<END>>'
} >"$reason"
[ "$(wc -c <"$reason" | tr -d ' ')" = 15000 ] || fail "the reason is not 15000 characters"

# A boom of the script's own: it stops each edit with the reason, and says
# nothing at session start.
cat >"$scratch/bin/boom" <<EOF
#!/bin/sh
cat >/dev/null
if [ "\$*" = "hook pre-tool-use" ]; then
  : >"$scratch/stopped"
  jq -cn --rawfile reason "$reason" '{systemMessage: "Boom: 1 rule for cap", hookSpecificOutput: {hookEventName: "PreToolUse", permissionDecision: "deny", permissionDecisionReason: \$reason}}'
fi
EOF
chmod +x "$scratch/bin/boom"

for k in $(env | grep -Eo '^(CLAUDE|ANTHROPIC_)[A-Z_]*=' | tr -d =); do
  unset "$k"
done

target=$scratch/work/cap.txt
API_LOG=$scratch/log API_PORT=$scratch/port API_TARGET=$target \
  bun "$here/api.mjs" 2>"$scratch/api.err" &
api_pid=$!
for _ in $(seq 100); do
  [ -s "$scratch/port" ] && break
  sleep 0.1
done
[ -s "$scratch/port" ] || fail "the API did not start: $(cat "$scratch/api.err")"

# The key is no credential: the API on 127.0.0.1 reads none.
if ! (cd "$scratch/work" &&
  HOME=$scratch/home PATH="$scratch/bin:$PATH" \
    CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1 \
    ANTHROPIC_BASE_URL="http://127.0.0.1:$(cat "$scratch/port")" \
    ANTHROPIC_API_KEY=not-a-key \
    claude -p "Write the file." --plugin-dir "$plugin" \
    --dangerously-skip-permissions --output-format text \
    </dev/null >"$scratch/claude.out" 2>"$scratch/claude.err"); then
  fail "claude failed: $(cat "$scratch/claude.err")"
fi

[ -e "$scratch/stopped" ] || fail "the plugin's PreToolUse hook did not run boom"
[ -e "$target" ] && fail "the file was written: the stop did not hold"

# The text of the tool_result of the Write, in the request after it.
result=$(jq -rs '[.[].body.messages[]?.content | arrays | .[]
  | select(.type? == "tool_result" and .tool_use_id == "toolu_cap")
  | if (.content | type) == "string" then .content else ([.content[]?.text? // empty] | join("")) end] | last // empty' \
  "$scratch"/log/*.json)
[ -n "$result" ] || fail "no request held the tool_result of the Write"
case "$result" in
*"$(cat "$reason")"*) ;;
*) fail "the agent did not get the whole reason: it got ${#result} characters, $(printf '%s' "$result" | head -c 200)" ;;
esac
case "$result" in
*"persisted-output"* | *"Full output saved to"*) fail "the reason went to a file" ;;
esac
echo "claude-code: cap: the agent got the whole reason of 15000 characters, in a tool_result of ${#result} characters"
