Cheminée
---

The chemistry search stack. Index chemical structures, including arbitrary extra properties. Fast to search and your callers don't need RDKit.

 * RDkit provides the smarts
 * Rust provides the type safety and relative ease of use

See [rdkit-sys](https://github.com/tureus/rdkit-sys) for more on how the bindings work.

Docker builds
---

The cross project has a nice cross compiler docker image but it uses the old Ubuntu 16.04. We should try upgrading to
Ubuntu 20.04 and see how it works for our case.

Logging in to ghcr.io:

    # generate a personal access token in your github settings
    echo ghp_123 | docker login ghcr.io -u your_gh_username --password-stdin

Cutting A New Release
---

    cargo release patch --tag-prefix='' --execute
