## Upgrading from 0.1.49

Back up your complete Corbanu home before upgrading, with Corbanu stopped.
Keep the configuration, databases (including any SQLite sidecar files), vault
and its key storage, sessions and history together.

This version adds upstream's state database migrations (Corbanu numbers
0054–0069) and new migrations to the memory, queue and thread history
databases. 0.1.49 refuses to open the upgraded state database and may report
that its migrations belong to a different distribution. This is a version
mismatch, not evidence of corruption. To return to 0.1.49, restore the matching
pre-upgrade backup of the whole home along with the old package. Do not delete
migration records to bypass the check. Conversations created after the backup
will not be present in that backup.
