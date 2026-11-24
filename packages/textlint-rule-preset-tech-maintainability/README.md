<!-- suikou lint applies to this file. Read "Writing this document" below before you edit it. -->
# textlint-rule-preset-tech-maintainability

textlint-rule-preset-tech-maintainability is a textlint preset that implements rules M1
through M7 of [suikou](https://github.com/KazuyoshiAkiyama/suikou).
Each rule detects one habit that makes a document hard to keep current. The preset
supports Japanese and English and identifies the language of each document by itself, so a
single preset works for a document in either language and for a repository that contains
documents in both.

[日本語](README.ja.md)

## What each rule detects

The rules are taken from the sections on lists, headings, and timeless documentation in
the Google developer documentation style guide. They address a problem that existed before
generative AI: whether or not a model wrote it, a document that violates these rules
becomes less trustworthy to its readers with every edit. The precise definition of each
rule is in [`docs/content/en/metrics.md`](../../docs/content/en/metrics.md) ("The M rules").

The rule key in the left column is the name this package registers internally. It also
matches the suffix of the corresponding Rust rule ID (`maint/list-lead-in` and so on, defined in
`crates/suikou-core/src/rules/mod.rs`). The ID that textlint prints for a finding has a
different prefix, which is described under "Installing" below.

| Rule key | Checks |
|---|---|
| `list-lead-in` | A list whose lead-in line is grammatically tied to the items, so the sentence breaks when an item is added or removed |
| `item-count` | Prose that states how many items there are, so the prose becomes wrong when an item is added or removed |
| `numbered-heading` | A heading with a section number typed by hand |
| `manual-number` | A bulleted list item whose text begins with a number typed by hand |
| `parallel-items` | A list whose items do not all end in the same form |
| `time-dependent` | A word whose meaning depends on when the document is read |
| `trailing-etc` | Prose or a list item that ends with "etc." or "など" |

## Installing

```sh
npm install --save-dev textlint textlint-rule-preset-tech-maintainability
```

Register the package in `.textlintrc` as follows.

```json
{
  "rules": {
    "preset-tech-maintainability": true
  }
}
```

A single rule can be disabled by setting it under this package's namespace.

```json
{
  "rules": {
    "preset-tech-maintainability": {
      "manual-number": false
    }
  }
}
```

When the package is loaded this way, findings are printed with the prefix
`tech-maintainability/`, as in
`tech-maintainability/list-lead-in`. textlint builds that prefix by removing
`preset-` from the config key above; the naming follows textlint's own convention and is not
chosen by this package. The `maint/` prefix that appears in this document belongs to `suikou check`,
and it is used here only so that the rule key after it (`list-lead-in` and so on) can be
matched against the table above. D-23 in
[`docs/content/en/decisions.md`](../../docs/content/en/decisions.md) explains in full
why the two prefixes differ.

## How this package relates to the Rust implementation

The core analysis of suikou is in a Rust crate
([`crates/suikou-core`](../../crates/suikou-core)) and is distributed as the `suikou` binary. This
package is a separate implementation of the same M rules for people who already use
textlint. The judgment logic and the regular expressions were ported line by line from
`crates/suikou-core/src/rules/mod.rs`; no part of this package was designed independently.

The two implementations differ in one structural respect. `suikou check` splits the raw source
into blocks, one block per physical line, with the line-based parser in
`markdown.rs`. This package does the same thing. Each rule receives the whole file through
`context.getSource()` and passes it to a JavaScript port of that line-based parser,
instead of traversing textlint's own Markdown AST. Without the port, a paragraph that soft-wraps
across two lines would become a single joined node inside textlint. A match against the
joined text could then include a phrase that crosses a line break the Rust side never joins.
Because of the port, both tools report the same number of findings, and the
[golden tests](../../tests/golden/) in this repository confirm that the counts match for each file.

The parser's preprocessing is ported step by step as well, including the treatment of Hugo
shortcodes that contain code. A `{{< highlight yaml >}}` block contains YAML comments, and a `#`
at the start of such a comment is read as a heading unless the block is removed first. Both
implementations remove `highlight`, `mermaid`, and any `tab` that declares `codelang`, and
both leave a shortcode untouched when its closing tag cannot be found.

Japanese morphological analysis is the other difference. suikou-core reads one dictionary
through lindera, while this package reads a different dictionary, IPADIC, through
[kuromojin](https://github.com/azu/kuromojin), the standard morphological analyzer among
textlint tools. The dictionary used by suikou-core gives suffixes, pronouns, and
adjectival noun stems separate top-level categories. IPADIC does not, and it puts all three
under a single noun category with finer subcategories. As a result, the taigen check that
`list-lead-in` and `parallel-items` use reduces to a single condition under IPADIC. D-23 in
[`docs/content/en/decisions.md`](../../docs/content/en/decisions.md) gives the reasoning
and the probe that confirmed it.

This package does not include the structural rules. `structure/missing-section` and the
related rules ask whether a section is missing from the document, and a missing section
has no node that a finding could be attached to. The document-wide metrics are excluded for
the same reason, as D-01 records. The `suikou` binary handles those rules.

The remaining difference is intentional and has nothing to do with structure: `suikou check`
always prints its explanations in Japanese, even when the document is in English. This
package prints explanations in the language it detects in the document, because a message
that no one present can read helps no one.

## Examples

A lead-in line that ends in a bound predicate breaks as soon as the list below it
changes:

```markdown
The files affected are:

- `/etc/app/main.conf`
- `/etc/app/conf.d/*.conf`
```

`list-lead-in` flags this lead-in line because it ends in a particle instead of a full
sentence or a noun before the colon. Rewriting the lead-in as a full sentence, such as
"The files affected are as follows:" or "The affected files are these:", clears the
finding with no change to the list.

A list whose items end in different forms breaks for the same reason:

```markdown
- Validate the syntax of the file.
- Reload the service
```

`parallel-items` flags this list because one item ends with a period and the other does
not. The finding is cleared when every item ends the same way, either all as full
sentences or all as phrases.

## Where this preset overlaps with others

[`@textlint-ja/textlint-rule-preset-ai-writing`](https://github.com/textlint-ja/textlint-rule-preset-ai-writing)
includes a rule named `no-ai-colon-continuation`. That rule also uses kuromojin to check
whether the text before a colon ends in a predicate, which is similar to the Japanese
check in `list-lead-in`. Even where the checks overlap, the two rules serve different
purposes. The ai-writing rule detects a phrasing pattern typical of AI-written Japanese, while
`list-lead-in` detects a list structure that breaks when an item is added or removed.

With both presets enabled, the same line can receive two findings. Neither preset
suppresses the other. A user who has enabled both can read both messages and decide which one
applies to the document.

## Testing

```sh
npm test
```

The tests run on Node's built-in test runner (`node --test`) instead of [`textlint-tester`](https://github.com/textlint/textlint-tester).
`textlint-tester` expects Mocha's global `describe` and `it`. Its fallback for other runners does not wait
for the Promise that a test case returns, so assertions passed before the lint they
relied on had finished. `test/support/lint.js` wraps `@textlint/kernel` directly instead,
in a way that waits correctly under `node --test`.

`test/golden.test.js` runs this package on every file in
[`tests/golden/input/`](../../tests/golden/input/) and checks that, for each rule, the
number of findings equals the number that `suikou check --format json` reports for the
same file. Matching those counts is the completion criterion for this package.

## License

You may use this package under either the MIT or the Apache-2.0 license.
