# The self-tests' text match, sourced by each one that checks output.
# A here-string ends in a newline, so an empty needle would match even an
# empty haystack: a case handed one is broken, so it stops the run rather
# than counting as a pass (#2847).

# contains NEEDLE HAYSTACK: succeeds when HAYSTACK holds NEEDLE as fixed text.
contains() {
  if [ -z "$1" ]; then
    echo "✗ a contains check was handed an empty needle, so it would match anything" >&2
    exit 2
  fi
  grep -qF -- "$1" <<<"$2"
}
