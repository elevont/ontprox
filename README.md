<!--
SPDX-FileCopyrightText: 2024 Robin Vobruba <hoijui.quaero@gmail.com>

SPDX-License-Identifier: CC0-1.0
-->

# `ontprox` - **Ont**ology **Prox**y

[![License: AGPL-3.0-or-later](
    https://img.shields.io/badge/License-AGPL--3.0--or--later-blue.svg)](
    LICENSE.txt)
[![REUSE status](
    https://api.reuse.software/badge/codeberg.org/elevont/ontprox)](
    https://api.reuse.software/info/codeberg.org/elevont/ontprox)
[![Repo](
    https://img.shields.io/badge/CodeBerg.org-green?style=flat&label=Repo)](
    https://codeberg.org/elevont/ontprox)
[![Package Releases](
    https://img.shields.io/crates/v/ontprox.svg)](
    https://crates.io/crates/ontprox)
[![Dependency Status](
    https://deps.rs/repo/github/elevont/ontprox/status.svg)](
    https://deps.rs/repo/github/elevont/ontprox)
[![Build Status](
    https://github.com/elevont/ontprox/workflows/build/badge.svg)](
    https://github.com/elevont/ontprox/actions)

RDF format conversion as a web-service.

## What

A tiny HTTP service that allows to fetch an [RDF] [ontology]
in a variety of different formats
(e.g. RDF/XML, JSON-LD, Turtle, ...),
as long as it is available in one of them.

## Why

This tool - when hosted - comes in handy
when trying to comply with [best practise of publishinig ontologies/vocabularies](
https://www.w3.org/TR/swbp-vocab-pub/#negotiation),
as part of that pracice suggests to use [content negotation]
to provide the ontology in different formats.
We could just statically convert our ontology into different formats,
and be done with it.
As ontologies can somehow be though of standards for communication,
it would be good if they'd be very stable in most cases.
In practice though - as live goes -
it is pretty unwise to consider them as static,
never changing entities of the digital world;
that would mean,
setting ourselfs up for failure, stagnation, ... project death.
No! Ontologies _have to_ change,
and in practice, they do.
Just as in software, they are best hosted on [VCS/SCM][VCS] like [git].
Naturally, they will also have versions - releases and latest development states,
all of which we might want to make available (under different [IRI/URI][URI]s),
_following best practise_ for publishing ontologies.
This is unfeasible/impractical to do by hosting statically converted files
for each revision for each RDF serialization format + HTML.
Thats where this tool enters the scene.

## How

As an example,
let us imagine an ontology is served under IRI/URI
<https://w3id.org/someorg/ont/thisone>
as Turtle.
This tool - hosted as a public service - can then be used as a proxy,
given this URI plus a target MIME-type
(through the HTTP `Accept` header) -
for example `application/ld+json` or `text/html`;
this MIME-type will then be served,
if the conversion is possible.

Internally we use [pyLODE] for conversion to HTML,
and the python [RDFlib] (through a thin CLI wrapper)
for all other conversion.

**NOTE**
Caching is involved!

## Usage

### Container

This tool is part of [RDF-tools]
(not to be confused with [rdftools]),
a [Docker] image, containing a collection of RDF related
CLI and web-service tools.
To use the tool alone/directly,
follow the steps in the next few sections.

### Prerequisites

You need to have [`pylode`][pyLODE]
and [`rdf-convert` (from the _rdftools_ tool set][rdftools]
installed and available on your [`PATH`][PATH].

### Install

As for now, you have two choices:

1. [Compile it](#how-to-compile) yourself
1. Download a Linux x86\_64 statically linked binary from
   [the latest release](https://codeberg.org/elevont/ontprox/releases/latest)

### Run

(see [Install](#install) and [Prerequisites](#prerequisites) first)

using the compiled binary:

```shell
$ ontprox
...
listening on 127.0.0.1:3000
```

or from source:

```shell
$ cargo run
...
listening on 127.0.0.1:3000
```

The available CLI arguments:

```text
$ ontprox --help
    A tiny HTTP service that allows to fetch an RDF ontology
    that is available in one/a few format(s),
    in others.


Usage: ontprox [OPTIONS]

Options:
  -V, --version
          Print version information and exit. May be combined with -q,--quiet, to really only output the version string.

  -v, --verbose
          more verbose output (useful for debugging)

  -q, --quiet
          Minimize or suppress output to stderr; stdout is never used by this program, with or without this option set.

  -p, --port <PORT>
          the IP port to host this service on

          [default: 3000]

  -a, --address <IP_ADDRESS>
          the IP address (v4 or v6) to host this service on

          [default: 127.0.0.1]

  -c, --cache-dir <DIR_PATH>
          a variable key-value pair to be used for substitution in the text

          [default: ~/.cache/ontprox]

  -C, --prefere-conversion
          Preffer conversion from a cached format over downloading the requested format directly from the supplied URI.

  -h, --help
          Print help (see a summary with '-h')
```

### Fetches

(see [Run](#run) first)

When the service is running on `http://127.0.0.1:3000`,
you can fetch the HTML version
of e.g. the [ValueFlows] (VF) ontology as HTML,
either by opening <http://127.0.0.1:3000?uri=https://w3id.org/valueflows/ont/vf.TTL>
in your browser, or on the command line with [CURL]:

```shell
curl "http://127.0.0.1:3000?uri=https://w3id.org/valueflows/ont/vf.TTL" \
    -H "Accept: text/html" \
    > ont_vf.html
```

To give an other example,
this fetches the [Open Know-How] (OKH) ontology as JSON-LD:

```shell
curl "http://127.0.0.1:3000?uri=https://w3id.org/oseg/ont/okh" \
    -H "Accept: application/ld+json" \
    > ont_okh.jsonld
```

## How to compile

You need to install Rust(lang) and Cargo.
On most platforms, the best way to do this is with [RustUp].

Then get the sources with:

```bash
git clone --recurse-submodules https://codeberg.org/elevont/ontprox.git
cd ontprox
```

Then you can compile with:

```bash
cargo build
```

If all goes well,
the executable can be found at `target/debug/ontprox`.

[CURL]: https://curl.se/
[Docker]: https://en.wikipedia.org/wiki/Docker_(software)
[ontology]: https://en.wikipedia.org/wiki/Ontology_(information_science)
[Open Know-How]: https://github.com/iop-alliance/OpenKnowHow
[PATH]: https://en.wikipedia.org/wiki/PATH_(variable)
[pyLODE]: https://github.com/RDFLib/pyLODE
[RDF]: https://www.w3.org/RDF/
[RDF-tools]: https://gitlab.com/OSEGermany/rdf-tools
[rdftools]: https://github.com/elevont/rdftools
[RDFlib]: https://rdflib.readthedocs.io
[RustUp]: https://rustup.rs/
[ValueFlows]: https://valueflo.ws/
[content negotation]: https://en.wikipedia.org/wiki/Content_negotiation
[VCS]: https://en.wikipedia.org/wiki/Version_control
[git]: https://git-scm.com/
[URI]: https://en.wikipedia.org/wiki/Uniform_Resource_Identifier
