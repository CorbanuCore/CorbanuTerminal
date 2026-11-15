## Windows sandbox

- The planted-folder watch stops reading a git file if it grows past the size
  observed on its open handle and flags it as changed. Unchanged files keep
  their existing fingerprints.
- Cancelling a non-admin command while it starts now terminates and waits for
  its job using the normal exit cleanup before restoring folder labels.
