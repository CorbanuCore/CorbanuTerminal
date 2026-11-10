## Windows sandbox

- **Default (elevated) sandbox: a test now covers a runner that can't
  start** (#1909). It checks that the error is reported and that the
  planted-folder watch ends on its own thread, not on the one that drives
  other tasks. No behaviour change.
