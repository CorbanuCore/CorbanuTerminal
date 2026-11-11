## Windows sandbox

- **Checked: the planted-folder watch doesn't follow a file re-marked as a
  link just before it reopens it** (#1909, both sandboxes). On Windows
  Server 2022 and 2025 and on Windows 11, reopening a file from its handle
  refuses to follow a link, so the opening raised in review isn't there. A
  new test keeps it that way. No behaviour change.
