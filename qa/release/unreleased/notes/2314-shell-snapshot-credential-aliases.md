## Security

- Shell snapshots now omit variables containing protected credential values, including aliases and concatenations created by shell startup files. Filtering happens before the snapshot file is written; credential-name filtering remains in place.
