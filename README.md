

download file may be not a tar file
---

At first, I assume all asset download files are some sort of compressed files, like `.tar`, `.tar.gz`, and `.zip`.
But some repositories put exectuable in assets directly. (for now, `nvim.appimage` and `tldr`)

So now, I need to decide how to distinguish them, because they need different ways to handle. (extract or not)

1. `asset_download_filename` ends with `.tar`, `.tar.gz`, and `.zip`
2. not satisfy conditions above and `asset_download_filename` is the same as `src`


