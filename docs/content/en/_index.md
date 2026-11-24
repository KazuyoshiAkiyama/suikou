---
title: "suikou"
doctype: "overview"
---

suikou is a linter and harness for technical documentation, covering both Japanese and
English.

Technical documentation produced by generative AI shows patterns that extend beyond
vocabulary. The text uses many figures of speech, keeps ending sentences the same way,
connects one sentence to the next clumsily, and includes so much detail that the document
becomes costly to maintain. suikou detects these patterns and then revises the document
toward a form that remains easy to maintain.

Each rule is grounded in either a measurement or a published standard.
None of them is based on intuition, because intuitions about style usually fail once they
are tested against data. One example is the coefficient of variation of sentence length.
It is often cited as a sign of AI-written text, yet the measured values fell within the
range of human writing.

## Getting started

To install, extract the archive published on GitHub Releases and place `suikou` in a
directory on your PATH. Day-to-day work needs only the commands below.

```sh
suikou plot docs/design.md --doctype design  # settle the structure before writing
suikou brief --doctype design                # paste into the prompt before generating
suikou check docs/design.md                  # check what was written
```

[Writing](writing/) explains the norms and steps to follow when writing a document, and
[Tuning](tuning/) explains how to adapt the tool to your own documents. Installation and a
description of each command are in the [design](design/) document. [Metrics](metrics/)
gives the precise definitions, and [decisions](decisions/) records what was decided and
the reasons behind each decision.
