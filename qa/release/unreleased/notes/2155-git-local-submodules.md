## Windows sandbox

- Git commands in a no-network sandbox now refuse all transports, including local-file submodule clones. Direct local clones, fetches and pushes are also refused; commands that work within an existing repository, such as status and diff, remain available. This closes the local-submodule exception introduced by the file-only transport allow-list (#2155).
