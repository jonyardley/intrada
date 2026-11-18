# Sourced by claim-issue.sh and crew-loop.sh (#2859): the owner's roadmap board.
# Only the owner's login can set it; the agent account has no access, so a loop
# session's claim leaves it to the loop, which sets it with the owner's login.

BOARD_OWNER="jonyardley"
board_number="2"
board_id="PVT_kwHOAAr6vs4A1_pq"
board_status_field="PVTSSF_lAHOAAr6vs4A1_pqzgrX99A"
board_status_in_progress="47fc9ee4"

# Sets issue $1 of $REPO to In progress. Fails when either call does.
board_in_progress() {
  local item
  item="$(gh project item-add "$board_number" --owner "$BOARD_OWNER" \
    --url "https://github.com/$REPO/issues/$1" --format json -q .id 2>/dev/null)" || return 1
  [ -n "$item" ] && gh project item-edit --id "$item" --project-id "$board_id" \
    --field-id "$board_status_field" \
    --single-select-option-id "$board_status_in_progress" >/dev/null 2>&1
}
