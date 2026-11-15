## Windows sandbox

The experimental Windows provisioning service remains off by default. Its setup
helper now refuses hard-linked and reparse-point files in sandbox directories,
opens leaves through verified directory handles, and applies inherited permissions
only to individually checked handles. If setup reports a linked file, remove the
extra link or restore a regular file before retrying. This does not enable the
service or change ordinary interactive sandbox setup. Existing nested cwd junctions
also cause refusal; activation must wait for an owner-approved cleanup or link
handling policy. No exception for those junctions is enabled here.
