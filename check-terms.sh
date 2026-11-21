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
# nothing write beside the store.
#
#   sh check-terms.sh head <out> --repository <name>
#     writes the store's last entry, as boom terms check reads it, to
#     <out>: the input of the terms task that names the store, so a pass
#     is replayed only for the entry it read
#   sh check-terms.sh check --repository <name> [--base <branch>]
#     checks the files of the change against <branch>, main where it is
#     not given
#
# BOOM names the boom to run; where it is not set, the script builds the
# tree's. Where there is no store, boom checks nothing and says so.
set -u

usage() {
  echo "usage: check-terms.sh head <out> --repository <name> | check --repository <name> [--base <branch>]" >&2
  exit 2
}

[ $# -ge 1 ] || usage
mode=$1
shift
out=
if [ "$mode" = head ]; then
  [ $# -ge 1 ] || usage
  out=$1
  shift
elif [ "$mode" != check ]; then
  usage
fi
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

if [ "$mode" = head ]; then
  answer=$("$boom" terms check --repository "$repository" --if-declared --json) || {
    echo "terms: boom could not read the store" >&2
    exit 1
  }
  mkdir -p "$(dirname "$out")"
  printf '%s\n' "$answer" | jq -c .head >"$out"
  exit 0
fi

# Where there is no store, no term is accepted.
answer=$("$boom" terms check --repository "$repository" --if-declared --json) || {
  echo "terms: boom could not read the store" >&2
  exit 1
}
if [ "$(printf '%s\n' "$answer" | jq -c .head)" = null ]; then
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
  echo "terms: the change adds or modifies no file to check"
  exit 0
fi
count=$(tr -cd '\0' <"$list" | wc -c | tr -d ' ')
xargs -0 "$boom" terms check --repository "$repository" --if-declared <"$list"
status=$?
case $status in
0) echo "terms: $count files of the change hold no old word of an accepted term of $repository" ;;
# xargs exits 123 where boom exits 1 to 125: boom names each old word.
123) echo "terms: the change writes an old word of an accepted term of $repository; write the term in its place, or mark a line that names it on purpose with boom: allow <old word>" >&2 ;;
*) echo "terms: boom terms check did not run, exit $status" >&2 ;;
esac
[ "$status" -eq 0 ]
