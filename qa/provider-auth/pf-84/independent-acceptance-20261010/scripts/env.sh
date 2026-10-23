S=/Volumes/CorbanuDrive/Corbanu/tmp/pf84acc-scratch
B=$S/bin/corbanu-main
# run candidate in a clean env with a disposable home: cx <home> args...
cx() { h="$1"; shift; mkdir -p "$h"; env -i HOME=$S/fakehome PATH=/usr/bin:/bin:/opt/homebrew/bin TERM=xterm-256color CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME="$h" "$B" "$@"; }
