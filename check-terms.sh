#!/bin/sh
# The terms task of the gate (term-check, "The gate checks the files that
# a change touches"): boom terms check, by the boom the tree builds, on
# each file that the change adds or modifies against the branch that it
# lands on, for the accepted terms of the repository. It skips the files
# under openspec/changes/archive/, the history of changes that landed,
# and the files that the change deletes. It reads each file for the old
# words of the accepted terms and for nothing else: the check of the
# term tool of its own words (no-task-reads-markdown). boom reads the
# store and writes nothing, so it runs where the gate's sandbox lets
# nothing write beside the store. The store is outside the tree, so the
# task never replays, and its last line names the entry of the store's
# log that it read.
#
#   sh check-terms.sh --repository <name> [--base <branch>]
#
# It checks the files of the change against <branch>, main where it is
# not given. BOOM names the boom to run; where it is not set, the script
# builds the tree's. Where there is no store, boom checks nothing and
# says so.
set -u

usage() {
  echo "usage: check-terms.sh --repository <name> [--base <branch>]" >&2
  exit 2
}

repository=
base=main
while [ $# -gt 0 ]; do
  case $1 in
  --repository)
    [ $# -ge 2 ] || usage
    repository=$2
    shift 2
    ;;
  --base)
    [ $# -ge 2 ] || usage
    base=$2
    shift 2
    ;;
  *) usage ;;
  esac
done
[ -n "$repository" ] || usage

# The boom the tree builds, by the Rust the tree pins.
boom=${BOOM:-}
if [ -z "$boom" ]; then
  if ! cargo build --locked -p boom >&2; then
    echo "terms: cargo build --locked -p boom failed" >&2
    exit 1
  fi
  target=$(cargo metadata --format-version 1 --no-deps --locked | jq -r .target_directory)
  boom=$target/debug/boom
fi
if [ ! -x "$boom" ]; then
  echo "terms: no boom at $boom" >&2
  exit 1
fi

# The entry of the store's log that the check reads. Where there is no
# store, no term is accepted.
answer=$("$boom" terms check --repository "$repository" --if-declared --json 2>/dev/null) || {
  echo "terms: boom could not read the store: $answer" >&2
  exit 1
}
entry=$(printf '%s\n' "$answer" | jq -r '.head.number // empty')
if [ -z "$entry" ]; then
  echo "terms: no store, so no term of $repository is accepted, and nothing is checked"
  exit 0
fi

# The files the change adds or modifies: each a file at its new path,
# none that it deletes, none of the archive.
if ! git rev-parse --verify --quiet "$base^{commit}" >/dev/null; then
  echo "terms: no branch $base to compare the change with" >&2
  exit 1
fi
fork=$(git merge-base HEAD "$base") || {
  echo "terms: HEAD and $base have no commit in common" >&2
  exit 1
}
list=$(mktemp "${SLUICE_TMP:-${TMPDIR:-/tmp}}/boom-terms.XXXXXX")
trap 'rm -f "$list"' EXIT
git diff --name-only --no-renames --diff-filter=AM -z "$fork" HEAD -- . \
  ':(exclude)openspec/changes/archive/**' >"$list"
if [ ! -s "$list" ]; then
  echo "terms: the change adds or modifies no file to check, at entry $entry of the store's log"
  exit 0
fi
count=$(tr -cd '\0' <"$list" | wc -c | tr -d ' ')
xargs -0 "$boom" terms check --repository "$repository" --if-declared <"$list" >/dev/null
status=$?
case $status in
0) echo "terms: $count files of the change hold no old word of an accepted term of $repository, at entry $entry of the store's log" ;;
# xargs exits 123 where boom exits 1 to 125: boom names each old word.
123) echo "terms: the change writes an old word of an accepted term of $repository, at entry $entry of the store's log; write the term in its place, or mark a line that names it on purpose with boom: allow <old word>" >&2 ;;
*) echo "terms: boom terms check did not run, exit $status" >&2 ;;
esac
[ "$status" -eq 0 ]
