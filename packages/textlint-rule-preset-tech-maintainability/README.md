<!-- suikou lint applies to this file. See "Writing this document" below before editing it. -->
# textlint-rule-preset-tech-maintainability

textlint-rule-preset-tech-maintainability is a textlint preset implementing rules M1 through M7 of [suikou](https://github.com/KazuyoshiAkiyama/suikou). Each rule detects one writing pattern that makes documentation difficult to keep accurate across edits. Because the preset supports both Japanese and English and identifies each document's language automatically, one preset covers single-language documents in either language as well as repositories containing documents in both languages.

[日本語](README.ja.md)

## Rules included

These rules come from the sections on lists, headings, and timeless documentation in the Google developer documentation style guide. They address a maintenance issue that predates generative AI: violating them repeatedly causes readers to lose trust in a document after edits, regardless of whether a human or a language model produced the text. Formal specifications for all rules appear in [`docs/content/en/metrics.md`](../../docs/content/en/metrics.md) under "The M rules".

The rule key in the left column is the internal identifier registered by this package and also serves as the suffix of the matching Rust rule ID (for example, `maint/list-lead-in` in `crates/suikou-core/src/rules/mod.rs`). textlint outputs findings under a different prefix, explained under "Installation" below.

| Rule key | Issue detected |
|---|---|
| `list-lead-in` | A list lead-in line grammatically coupled to the list items, where adding or removing an item makes the sentence ungrammatical |
| `item-count` | Prose that states the number of items, becoming inaccurate whenever an item is added or removed |
| `numbered-heading` | A heading containing a manually written section number |
| `manual-number` | A bulleted list item whose text opens with a manually written number |
| `parallel-items` | A list whose items do not share one consistent ending form |
| `time-dependent` | A word whose meaning depends on when a reader views the document |
| `trailing-etc` | Prose or a list item ending with "etc." or "など" |

## Installation

```sh
npm install --save-dev textlint textlint-rule-preset-tech-maintainability
```

Enable the preset in `.textlintrc`:

```json
{
  "rules": {
    "preset-tech-maintainability": true
  }
}
```

Disable individual rules through the preset's configuration namespace:

```json
{
  "rules": {
    "preset-tech-maintainability": {
      "manual-number": false
    }
  }
}
```

When configured this way, textlint outputs findings with the `tech-maintainability/` prefix, such as `tech-maintainability/list-lead-in`. textlint constructs that prefix automatically by stripping `preset-` from the configuration key above rather than using a prefix selected by this package. The `maint/` prefix used in this document belongs to `suikou check` and appears here solely so that the rule key after the slash (for example, `list-lead-in`) corresponds directly to the table above. D-23 in [`docs/content/en/decisions.md`](../../docs/content/en/decisions.md) explains why the two prefixes differ.

## Comparison with the Rust implementation

suikou's core analyzer is implemented in a Rust crate ([`crates/suikou-core`](../../crates/suikou-core)) and distributed as the `suikou` binary. This package provides a second, independent implementation of the same M rules for projects already running textlint. Its detection logic and regular expressions are ported line by line from `crates/suikou-core/src/rules/mod.rs`, without newly invented rules.

A notable structural design choice concerns how Markdown parsing works across both implementations. `suikou check` parses raw source text directly into one block per physical line using the line-based parser in `markdown.rs`. Rather than traversing textlint's built-in Markdown AST, this package follows the same approach: each rule reads the full file through `context.getSource()` and passes it to a JavaScript port of that line-based parser. Traversing textlint's AST would merge a paragraph soft-wrapped across two lines into one joined node, potentially matching phrases across line breaks that the Rust parser never joins. Porting the line-based parser ensures identical finding counts between the two tools, verified file by file by the [golden tests](../../tests/golden/) in this repository.

Preprocessing steps in the parser are ported identically, including Hugo shortcodes whose bodies contain code. A `{{< highlight yaml >}}` block can contain YAML comments whose leading `#` would be parsed as a Markdown heading unless the block is removed first. Both implementations remove `highlight`, `mermaid`, and any `tab` shortcode specifying `codelang`, while leaving a shortcode unchanged whenever its closing tag is missing.

Japanese morphological analysis differs between the two implementations. suikou-core analyzes text using one dictionary through lindera, whereas this package uses IPADIC through [kuromojin](https://github.com/azu/kuromojin), the standard morphological analyzer for textlint. The dictionary used by suikou-core distinguishes suffixes, pronouns, and adjectival noun stems at the top part-of-speech level, while IPADIC groups all three under one noun category with finer subcategories. As a result, the taigen check used by `list-lead-in` and `parallel-items` simplifies to a single condition under IPADIC. D-23 in [`docs/content/en/decisions.md`](../../docs/content/en/decisions.md) records the rationale and the verification probe confirming this behavior.

Structural rules are omitted from this package. `structure/missing-section` and related structural rules check whether a required section is absent from a document, and an absent section provides no node to report a lint finding against. Document-wide metrics are omitted for the same reason, as documented in D-01. The `suikou` binary provides those rules instead.

Output language is another intentional difference: `suikou check` always prints explanations in Japanese, even when checking an English document. This package selects the explanation language to match the detected document language so that readers can understand the reported messages.

## Examples

A lead-in line ending in a grammatically bound predicate becomes ungrammatical when the list beneath it changes:

```markdown
The files affected are:

- `/etc/app/main.conf`
- `/etc/app/conf.d/*.conf`
```

`list-lead-in` flags the lead-in line above because it ends in a particle rather than a complete sentence or a noun before the colon. Rewriting the lead-in as a complete sentence—for example, "The files affected are as follows:" or "The affected files are these:"—resolves the finding without changing the list itself.

A list whose items do not share one consistent ending form causes a similar problem:

```markdown
- Validate the syntax of the file.
- Reload the service
```

`parallel-items` flags this list because one item ends with a period while the other does not. Giving every item the same ending form—either all complete sentences or all phrases—resolves the finding.

## Overlap with other presets

[`@textlint-ja/textlint-rule-preset-ai-writing`](https://github.com/textlint-ja/textlint-rule-preset-ai-writing) provides a rule called `no-ai-colon-continuation`. Like the Japanese half of `list-lead-in`, that rule uses kuromojin to detect whether text preceding a colon ends in a predicate. Even where the checks overlap, their goals differ: that rule targets a phrasing pattern typical of AI-generated Japanese, whereas `list-lead-in` detects list structures that become ungrammatical when items are added or removed. Enabling both presets can produce two findings on the same line. Neither preset suppresses the other, allowing users who enable both to review both messages and decide which applies to their document.

## Testing

```sh
npm test
```

Tests execute under Node's built-in test runner (`node --test`) instead of [`textlint-tester`](https://github.com/textlint/textlint-tester). `textlint-tester` relies on Mocha's global `describe` and `it` functions, and its fallback for other runners fails to await the Promise returned by each test case, allowing assertions to pass before linting finishes. To avoid this, `test/support/lint.js` wraps `@textlint/kernel` directly so that test execution awaits cleanly under `node --test`.

`test/golden.test.js` runs this package against every file in [`tests/golden/input/`](../../tests/golden/input/) and verifies that the finding count per rule matches what `suikou check --format json` outputs for the same file. Matching those counts serves as the completion criterion for this package.

## License

Dual-licensed under MIT or Apache-2.0, at your option.
