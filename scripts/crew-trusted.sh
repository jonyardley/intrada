# Sourced by the crew scripts (#2821): the logins whose comments the crew reads,
# the owner (the repository's owner) and the agent account its sessions act as
# (ostinato#11). Intrada is public, so anyone else's comment could steer a
# session. CREW_AGENT_LOGIN names another agent account.

CREW_TRUSTED="${REPO%%/*} ${CREW_AGENT_LOGIN:-ostinato-agent}"

# A jq definition, for gh's `--jq`, which takes no `--arg`: `trusted` holds for a
# comment by one of those logins, and fails for one with no author.
trusted_jq() {
  printf 'def trusted: (.author.login // "") as $a | %s | any(.[]; . == $a);' \
    "$(printf '%s\n' $CREW_TRUSTED | jq -R . | jq -sc .)"
}
