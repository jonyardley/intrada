#!/usr/bin/env bash
# Xcode caching (#2926) for the derived data the test build and ios-run-sim share, kept outside ios/
# so it never feeds the inputs fingerprint and passed on the command line; INTRADA_XCODE_CACHE=0 turns it off.

# Fills the array IOS_COMPILE_CACHE_ARGS, empty when caching is off.
# Expand it as ${IOS_COMPILE_CACHE_ARGS[@]+"${IOS_COMPILE_CACHE_ARGS[@]}"} so an
# empty array survives `set -u` on the system bash.
# shellcheck disable=SC2034 # read by the script that sources this file
ios_compile_cache_args() {
    IOS_COMPILE_CACHE_ARGS=()
    [ "${INTRADA_XCODE_CACHE:-1}" = "0" ] && return 0
    local root
    root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
    IOS_COMPILE_CACHE_ARGS=(
        COMPILATION_CACHE_ENABLE_CACHING=YES
        "COMPILATION_CACHE_CAS_PATH=$root/.xcode-cache"
    )
}
