## `/security` wording

- **Revoke all names its real scope.** After "Revoke all active authority",
  the confirmation now says it was revoked "in every session of this Corbanu
  Terminal on this Corbanu home", as its review says, instead of "in this
  session". A saved kill switch, on or off, also says that other Corbanu
  Terminals on the home take it within about a second (#1035).
- **A stale inspector view is not called degraded.** When the inspector's facts
  are more than 30 seconds old and nothing else is wrong, its top line reads
  "◌ Moderate set; this view is stale (status observed 33s ago)" instead of
  "▲ Moderate degraded"; press `r` to refresh. A stale view still never shows
  the green "Protected" badge (#1036).
