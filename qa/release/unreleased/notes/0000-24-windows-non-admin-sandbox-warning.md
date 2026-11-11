## Windows non-admin sandbox warning

- **The warning now matches what the non-admin sandbox does** (#1506, D264). It
  no longer says commands can delete or move files you own outside your
  workspace; since #705 they can't, except in low-integrity places. It now says
  what it doesn't stop: reading your files, network access by programs that
  ignore proxy settings, and a few ways of starting programs outside its job.
