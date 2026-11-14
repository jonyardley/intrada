# Sourced by the crew scripts (#2821): the logins whose comments the crew reads,
# the owner (the repository's owner) and the agent account its sessions act as
# (ostinato#11). Intrada is public, so anyone else's comment could steer a
# session. CREW_AGENT_LOGIN names another agent account.

CREW_TRUSTED="${REPO%%/*} ${CREW_AGENT_LOGIN:-ostinato-agent}"

# jq definitions, for gh's `--jq`, which takes no `--arg`. `trusted` holds for a
# comment by one of those logins, and fails for one with no author.
# `trusted_body` holds for an issue opened and last edited by one of them, since
# an issue's author can edit its body after the owner approved it.
trusted_jq() {
  printf 'def trusted_login: . as $a | %s | any(.[]; . == $a);
def trusted: (.author.login // "") | trusted_login;
def trusted_body: trusted and (.editor == null or ((.editor.login // "") | trusted_login));' \
    "$(printf '%s\n' $CREW_TRUSTED | jq -R . | jq -sc .)"
}
