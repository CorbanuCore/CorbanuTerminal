## Security

- Shell snapshots now also omit any exported variable that holds a copy of a credential the credential broker already brokers, even when a startup file unset the original variable (for example `HIDDEN_HEADER="$GH_TOKEN"` followed by `unset GH_TOKEN`). The broker checks the text itself, including credentials held only by the isolated broker process, without handing raw values out; if the check cannot be completed, no snapshot is written.
