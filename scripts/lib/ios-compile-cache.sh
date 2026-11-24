#!/usr/bin/env bash
# Xcode compilation caching for the Debug simulator builds (#2926): sourced by
# `_ios-build-for-testing` and `scripts/ios-run-sim.sh`, which share one
# derived data folder, so both must agree or each switch rebuilds everything.
# On by default; INTRADA_XCODE_CACHE=0 turns it off. The cache sits at the
# worktree root, outside ios/build so a clean keeps it and outside ios/ so it
# never feeds the inputs fingerprint. Passed on the command line so the
# package targets get it too. A clean test build took 25s with it and 40s
# without on 24 November 2026; a no change rebuild was the same either way.

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
