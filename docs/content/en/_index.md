---
title: "suikou"
doctype: "overview"
---

suikou is a documentation linter and authoring harness for Japanese and English technical writing.

Technical documentation produced by generative AI exhibits recurring issues beyond vocabulary. Common problems include reliance on figurative language, repetitive sentence endings, awkwardly connected sentences, and explicitly stating details that make ongoing maintenance difficult. suikou detects these patterns and helps make the documentation maintainable.

Every rule is based on empirical measurement or a written standard rather than intuition. Assumptions about writing style usually fail when tested empirically. For example, although the coefficient of variation of sentence length is widely cited as an indicator of AI-generated text, measurement showed that it overlaps with the human range.

## Getting Started

Extract the archive from GitHub Releases and place `suikou` on your PATH to install the tool. The following commands illustrate the standard workflow.

```sh
suikou plot docs/design.md --doctype design  # settle the structure before writing
suikou brief --doctype design                # paste into the prompt before generating
suikou check docs/design.md                  # check what was written
```

See [Writing](writing/) for authoring standards and procedures using suikou, and [Tuning](tuning/) for adapting it to your own documents. Complete installation instructions and descriptions of each command appear in [Design](design/). Exact metric definitions are documented in [Metrics](metrics/), and [Decisions](decisions/) records what was decided and why.
