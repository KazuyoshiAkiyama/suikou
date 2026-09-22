---
title: "suikou"
doctype: "overview"
---

suikou is a linter and a harness for technical documentation. It handles Japanese and
English.

Technical documentation written by generative AI carries habits that go beyond word
choice. It leans on figures of speech, it repeats the same sentence endings, it joins
sentences awkwardly, and it spells out details that make the document hard to maintain.
suikou finds these habits in the document, and it moves the document toward a shape that
stays maintainable.

Every rule here rests on a measurement or on a written standard.
Not one rule came from a hunch, because a hunch about style turns out to be wrong most of
the time once someone measures it. For example, the coefficient of variation of sentence
length is widely described as a marker of AI prose, and measurement put it on top of the
human range.

## Usage

Install it by unpacking the archive from GitHub Releases and putting `suikou` on the
PATH. The commands below cover the ordinary workflow.

```sh
suikou plot docs/design.md --doctype design  # settle the structure before writing
suikou brief --doctype design                # paste into the prompt before generating
suikou check docs/design.md                  # check what was written
```

The [design](design/) document covers installation and each command in turn.
[Metrics](metrics/) holds the exact definitions, and [decisions](decisions/) holds the
record of what was settled and why.
