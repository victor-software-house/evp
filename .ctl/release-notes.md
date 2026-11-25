EVP built from the `downstream` patch stack of victor-software-house/evp, a fork of [HalFrgrd/evp](https://github.com/HalFrgrd/evp).
PATCHES.md in the archive lists each patch: the macOS-style pointer, terminal-accurate text, eased pointer motion, the key pill and click choreography.

Apple Silicon macOS only. Install with mise:

```sh
mise use -g 'github:victor-software-house/evp[version_prefix=evp-]'
```

The archive also contains the README, the bundled evp skill and `examples/quickstart.tape`.
