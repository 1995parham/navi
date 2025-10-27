#!/usr/bin/env bash
set -euo pipefail

export NAVI_HOME
NAVI_HOME="$(cd "$(dirname "$0")/.." && pwd)"
source "${NAVI_HOME}/tests/core.sh"

export TEST_CHEAT_PATH="${NAVI_HOME}/tests/no_prompt_cheats"
export NAVI_EXE="${NAVI_HOME}/target/debug/navi"

if ! command_exists navi; then
	navi() {
		"$NAVI_EXE" "$@"
	}
	export -f navi
fi

_navi() {
	stty sane || true
	local path="${NAVI_TEST_PATH:-$TEST_CHEAT_PATH}"
	path="${path//$HOME/~}"
	export NAVI_ENV_VAR_PATH="$path"
	RUST_BACKTRACE=1 "$NAVI_EXE" --path "$NAVI_ENV_VAR_PATH" "$@"
}

_navi_cases() {
	local -r filter="${1::-2}"
	_navi --query "$filter" --best-match
}

_navi_cases_test() {
	_navi_cases "$1" |
		test::equals "$2"
}

_get_all_tests() {
	grep '^#' "${TEST_CHEAT_PATH}/cases.cheat" |
		grep ' ->' |
		sed 's/\\n/'"$(printf "$NEWLINE_CHAR")"'/g' |
		sed -E 's/# (.*) -> "(.*)"/\1|\2/g'
}

_get_tests() {
	local -r filter="$1"

	if [ -n "$filter" ]; then
		_get_all_tests |
			grep "$filter"
	else
		_get_all_tests
	fi
}

_navi_tldr() {
	_navi --tldr docker --query ps --print --best-match |
		test::contains "docker ps"
}

_navi_cheatsh() {
	_navi --cheatsh docker --query remove --print --best-match |
		test::contains "docker container prune"
}

_navi_widget() {
	local -r out="$(_navi widget "$1")"
	if ! echo "$out" | grep -q "navi "; then
		echo "$out"
		return 1
	fi
}

_navi_cheatspath() {
	_navi info cheats-path |
		grep -q "/cheats"
}

if ! command_exists fzf; then
	export PATH="$PATH:$HOME/.fzf/bin"
fi

cd "$NAVI_HOME"

filter="${1:-}"

# TODO: remove this
if [[ $filter == "_navi" ]]; then
	shift
	_navi "$@"
	exit 0
fi

test::set_suite "cases"
ifs="$IFS"
IFS=$'\n'
for i in $(_get_tests "$filter"); do
	IFS="$ifs"
	query="$(echo "$i" | cut -d'|' -f1)"
	expected="$(echo "$i" | tr "$NEWLINE_CHAR" '\n' | cut -d'|' -f2)"
	test::run "$query" _navi_cases_test "$query" "$expected"
done

test::set_suite "info"
test::run "cheats_path" _navi_cheatspath

test::set_suite "widget"
test::run "bash" _navi_widget "bash"
test::run "zsh" _navi_widget "zsh"
test::run "fish" _navi_widget "fish"
test::run "elvish" _navi_widget "elvish"
test::run "nu" _navi_widget "nushell"

# test::set_suite "3rd party"
# test::run "tldr" _navi_tldr
# test::run "cheatsh" _navi_cheatsh

test::finish
