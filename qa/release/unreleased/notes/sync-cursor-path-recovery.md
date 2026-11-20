## Fixes

- Cursor session import now recovers project folders whose path has several hyphenated or punctuated folder names, within a fixed limit of filesystem checks. A path that matches more than one folder, or needs more checks than the limit, is still left without a working folder.
