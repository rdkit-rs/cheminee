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
endpoints for "identity search" (i.e. exact structure match), "substructure search", and "superstructure search". We
plan to create a Tanimoto-based "similarity search" endpoint in the near future. Aside from structure searching, the API
also supports endpoints for standardization
of SMILES in bulk, as well as molecular format conversions (e.g. smiles-to-molblock and molblock-to-smiles). Indexing
endpoints (e.g. index creation, bulk indexing, compound deletion, index deletion) are
executed using Tantivy. Users can utilize the "basic search" (i.e. non-structure search) endpoint to search for
compounds by RDKit chemical descriptors (e.g. "exactmw", "NumAtoms",
etc.) or any other metadata you decided to include when indexing.

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

     cargo run -- basic-search -i "/tmp/cheminee/index0" -q "exactmw: [10 TO 10000] AND NumAtoms: [8 TO 100]" -l 10

Here, "i" refers to the index path, "q" refers to a composite query of chemical descriptor values and/or other indexed
data types, and "l" refers to the number of desired results (defaulted to 1000).

Substructure Search

For example:

    cargo run -- substructure-search -i /tmp/cheminee/index0 -s CCC -r 10 -t 10 -u true -e "exactmw: [20 TO 200]"

Similar to basic search, "i" refers to the index path. "s" refers to the query SMILES, "r" refers to the number of
desired results (defaulted to 1000), "t" refers to the number of tautomers to be used for the query SMILES if
applicable (defaulted to 0), "u" dictates whether to use indexed scaffolds to speed up the search (defaulted to "true"),
and "e" refers to the "extra query" which is a composite query for chemical descriptors or other index data types as in
the basic search implementation.

Superstructure Search

For example:

    cargo run -- superstructure-search -i /tmp/cheminee/index0 -s c1ccccc1CCc1ccccc1CC -r 10 -t 10 -u true -e "exactmw: [20 TO 200]"

The input arguments here are the same as used for substructure search.

Identity Search

For example:

    cargo run -- identity-search -i /tmp/cheminee/index0 -s c1ccccc1CC -e "exactmw: [20 TO 200]" -u true

Note: for identity search there is no need to specify the number of desired results (we are assuming you only want one
result), nor is there any need to specify any tautomers (we will look for the canonical tautomer of the query). Note:
the extra query above isn't necessary in this case, but if you have quasi-duplicate compound records (e.g. same
molecule, but some differing metadata), you can use the extra query to get more specific, otherwise Cheminée will stop
searching for the query once it finds an exact structure match (even if there are other duplicate structures present in
the
database).

Testing in Docker
---

Run the Cheminée image in a docker container. Note: in this command, logging is turned off, so you won't be able to
follow along during the SDF indexing step below. If you prefer to follow along, then replace "RUST_LOG=off" with "
RUST_LOG=info" but note that performance will be a bit slower:

    docker run --rm -dt -e RUST_LOG=off -p 4001:4001 --name cheminee ghcr.io/rdkit-rs/cheminee:0.1.30

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
very soon after you start the indexing, you might end up with no indexed compounds. If you want to follow along and kill
early for some simple testing, replace "RUST_LOG=off" with "RUST_LOG=info" in the docker run command above. Once you see
a
statement such as "10000 compounds processed so far" and you are happy with the number, then feel free to interrupt the
indexing:

    cheminee index-sdf -s tmp/sdfs/Compound_000000001_000500000.sdf.gz -i tmp/cheminee/index0

Go to "localhost:4001" in your favorite browser to test out the API endpoints. Note: for this test case, use "index0"
for the index
fields. Alternatively, test out the CLI some more:

    cheminee -h
    cheminee <CLI action> -h

Cutting A New Release
---

    cargo release patch --tag-prefix='' --execute
