#!/usr/bin/env bash

# Logging functions
log::ansi() {
	local bg=false
	local color=""
	local mod=""

	case "$@" in
	*reset*)
		echo -e "\e[0m"
		return 0
		;;
	*black*) color=30 ;;
	*red*) color=31 ;;
	*green*) color=32 ;;
	*yellow*) color=33 ;;
	*blue*) color=34 ;;
	*purple*) color=35 ;;
	*cyan*) color=36 ;;
	*white*) color=37 ;;
	esac
	case "$@" in
	*regular*) mod=0 ;;
	*bold*) mod=1 ;;
	*underline*) mod=4 ;;
	esac
	case "$@" in
	*background*) bg=true ;;
	*bg*) bg=true ;;
	esac

	# Return empty if no color was set
	if [[ -z "$color" ]]; then
		return 0
	fi

	if $bg; then
		# Background colors are 40-47, not 30-37
		echo -e "\e[$((color + 10))m"
	else
		echo -e "\e[${mod:-0};${color}m"
	fi
}

_log() {
	local template="$1"
	shift
	echo >&2 "$(printf "$template" "$@")"
}

log::success() { _log "$(log::ansi green)✔ %s$(log::ansi reset)\n" "$@"; }
log::error() { _log "$(log::ansi red)✖ %s$(log::ansi reset)\n" "$@"; }
log::warning() { _log "$(log::ansi yellow)➜ %s$(log::ansi reset)\n" "$@"; }
log::note() { _log "$(log::ansi blue)%s$(log::ansi reset)\n" "$@"; }

command_exists() {
	type "$1" &>/dev/null
}

NEWLINE_CHAR="\036"

PASSED=0
FAILED=0
SKIPPED=0
SUITE=""

test::set_suite() {
	SUITE="$*"
}

test::success() {
	PASSED=$((PASSED + 1))
	log::success "Test passed!"
}

test::fail() {
	FAILED=$((FAILED + 1))
	log::error "Test failed..."
	return
}

test::skip() {
	echo
	log::note "${SUITE:-unknown} - ${1:-unknown}"
	SKIPPED=$((SKIPPED + 1))
	log::warning "Test skipped..."
	return
}

test::run() {
	echo
	log::note "${SUITE:-unknown} - ${1:-unknown}"
	shift
	"$@" && test::success || test::fail
}

test::_escape() {
	tr '\n' "$NEWLINE_CHAR" | sed -E "s/[\s$(printf "$NEWLINE_CHAR") ]+$//g"
}

test::equals() {
	local -r actual="$(cat)"
	local -r expected="${1:-}"

	local -r actual2="$(echo "$actual" | test::_escape)"
	local -r expected2="$(echo "$expected" | test::_escape)"

	if [[ "$actual2" != "$expected2" ]]; then
		log::error "Expected '${expected}' but got '${actual}'"
		return 2
	fi
}

test::contains() {
	local -r haystack="$(cat)"
	local -r needle="${1:-}"

	local -r haystack2="$(echo "$haystack" | test::_escape)"
	local -r needle2="$(echo "$needle" | test::_escape)"

	if [[ "$haystack2" != *"$needle2"* ]]; then
		log::error "Expected '${haystack}' to include '${needle2}'"
		return 2
	fi
}

test::finish() {
	echo
	if [ $SKIPPED -gt 0 ]; then
		log::warning "${SKIPPED} tests skipped!"
	fi
	if [ $FAILED -gt 0 ]; then
		log::error "${PASSED} tests passed but ${FAILED} failed... :("
		return "${FAILED}"
	else
		log::success "All ${PASSED} tests passed! :)"
		return 0
	fi
}
