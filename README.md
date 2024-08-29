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

Cheminée is intended to work via CLI (e.g. for diagnostics) and API endpoints. "Substructure search" is
the first intended functionality, but Cheminée will eventually support "superstructure search", "similarity search",
and exact matches. Aside from structure searching, the API also supports standardization of SMILES in bulk as
well as indexing via Tantivy. Users can also search for compounds by RDKit chemical descriptors (e.g. "exactmw", "
NumAtoms", etc).


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

Testing in Docker
---

Run the Cheminée image in a docker container:

    docker run --rm -dt -p 4001:4001 --name cheminee ghcr.io/rdkit-rs/cheminee:0.1.30

Exec into the container:

    docker exec -it cheminee bash

Check out the CLI endpoints:

    cheminee -h

Fetch some PubChem SDFs for testing. Each file has ~400K+ compounds; use <ctrl + c> to stop when you're happy with
the number of files:

    mkdir -p tmp/sdfs
    cheminee fetch-pubchem -d tmp/sdfs

Create an index. We only have one schema at the moment (i.e. "descriptor_v1"):

    cheminee create-index -i tmp/cheminee/index0 -n descriptor_v1 -s exactmw

Start indexing an SDF file. Note: Cheminée does a bulk write after every 1,000 compounds. So if you <ctrl + c>
interrupt
very soon after starting the indexing, you might end up with no indexed compounds. If you want to follow along and kill
early for some simple testing, use "
RUST_LOG=info". Once you see a
statement such as "10000 compounds processed so far" and you are happy with the number, then feel free to interrupt the
indexing:

    RUST_LOG=info cheminee index-sdf -s tmp/sdfs/Compound_000000001_000500000.sdf.gz -i tmp/cheminee/index0

Or omit the "RUST_LOG=info" if you want better performance and you plan to let it finish.

Go to "localhost:4001" in your favorite browser to test out the API endpoints. Note: for this test case, use "index0"
for the index
fields. Alternatively, test out the CLI some more:

    cheminee -h
    cheminee <CLI action> -h

Cutting A New Release
---

    cargo release patch --tag-prefix='' --execute
