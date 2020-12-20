Executable upgrade policy
---
(draft)
executable_upgrade_policy:
- skip_when_exist: if executable exist, skip downloading.
- upgrade: if executable is older than latest release, upgrade.
- always

So for this policy, we need to check the executable status
- not exist: download latest anyway
- exist but old
  - for 'skip_when_exist': skip
   for 'upgrade': download
- exist and latest: skip downloading



download only return the target executable 
---

I want the `download` function to abstract whether the executable compressed or not, just return the executable directly.
So I only need to `copy` it into the destination.

Abstraction among `.tar`, `.tar.gz`, and `.zip`
---

I wanna create a abstraction among all these different compression and even no compression (maybe).


download file may be not a tar file
---

At first, I assume all asset download files are some sort of compressed files, like `.tar`, `.tar.gz`, and `.zip`.
But some repositories put exectuable in assets directly. (for now, `nvim.appimage` and `tldr`)

So now, I need to decide how to distinguish them, because they need different ways to handle. (extract or not)

1. `asset_download_filename` ends with `.tar`, `.tar.gz`, and `.zip`
2. not satisfy conditions above and `asset_download_filename` is the same as `src`

