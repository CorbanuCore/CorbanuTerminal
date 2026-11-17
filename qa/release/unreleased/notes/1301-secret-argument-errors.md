## Security

`corbanu login` and `corbanu account` usage errors no longer repeat an
unexpected value that might be a key or token accidentally supplied on the
command line. Supply credentials through stdin; the diagnostic also reminds
you that an accidentally typed key may remain in shell history (#1301).
