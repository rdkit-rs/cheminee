Cheminée
---

The chemistry search stack. Index chemical structures and include arbitrary extra properties. Fast to search and your
callers don't need RDKit.

* RDKit provides the smarts
* Rust provides the type safety and relative ease of use

See [rdkit-sys](https://github.com/tureus/rdkit-sys) for more on how the bindings work.

Tested on Rust Stabe 1.76 (`rustup default 1.76`)

Intended Functionality
---

Cheminée is intended to work via CLI (e.g. for diagnostics) and API endpoints. Cheminée currently supports CLI and API
endpoints for "identity search" (i.e. exact structure match), "substructure search", "superstructure search", and
Tanimoto-based "similarity search". Aside from structure searching, the API also supports endpoints for standardization
of SMILES in bulk, as well as molecular format conversions (e.g. smiles-to-molblock and molblock-to-smiles). Indexing
endpoints (e.g. index creation, bulk indexing, compound deletion, index deletion) are
executed using Tantivy. Users can also search for compounds by RDKit chemical descriptors (e.g. "exactmw", "
NumAtoms", etc.) using the "basic search" endpoint.


The API
---

To boot up the API server, you can run the following:

    cargo run --color=always --release --package cheminee --bin cheminee -- rest-api-server

The simplest way to test out the API is by going to localhost:4001 in your browser and testing out the
functionality of the different endpoints.

This repo is also set to automatically update a ruby gem whenever a new release is published. That gem is "
cheminee-ruby".
This gem is particularly helpful for interacting with the API in a ruby-friendly, programmatic environment.

See [cheminee-ruby](https://github.com/rdkit-rs/cheminee-ruby) for more information into how the gem works.

The CLI
---

Assuming compounds are already present in an index path, you can search by:

Basic Search

For example:

     cargo run -- search -i "/tmp/cheminee/scaffolds-index0" -q "exactmw: [10 TO 10000] AND NumAtoms: [8 TO 100]" -l 10

Here, "i" refers to the index path, "q" refers to the composite query of chemical descriptor values, and "l" refers
to the number of desired results.

Substructure Search

For example:

    cargo run -- substructure-search -i /tmp/cheminee/scaffolds-index0 -s CCC -r 10 -t 10 -e "exactmw: [20 TO 200]"

Similar to basic search, "i" refers to the index path, "s" refers to the query SMILES, "r" refers to the number of
desired results,
"t" refers to the number of tautomers to be used for the query SMILES (if applicable), and "e" refers to the
"extra query" which is a composite query for chemical descriptors as in the basic search implementation.

Superstructure Search

For example:

    cargo run -- superstructure-search -i /tmp/cheminee/scaffolds-index0 -s c1ccccc1CCc1ccccc1CC -r 10 -t 10 -e "exactmw: [20 TO 200]"

The input arguments here are the same as used for substructure search.

Similarity Search

For example:

    cargo run -- similarity-search -i /tmp/cheminee/scaffolds-index0 -s c1ccccc1CCc1ccccc1CCF -r 10 -t 10 -b 10 -e "exactmw: [20 TO 200]"

The additional "b" argument here refers to the maximum number of 6-dimensional PC bins that will be searched to extract
similar compounds. Currently, 46,656 (or 6^6) bins are constructed for this purpose.


Cutting A New Release
---

    cargo release patch --tag-prefix='' --execute
