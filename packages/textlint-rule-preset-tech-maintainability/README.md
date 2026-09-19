<!-- suikou lint applies to this file. See "Writing this document" below before editing it. -->
# textlint-rule-preset-tech-maintainability

A textlint preset for rules M1 through M7 of [suikou](https://github.com/KazuyoshiAkiyama/suikou).
Each rule catches one habit that makes a document hard to keep up to date later. The
preset handles Japanese and English, and it detects the language of each document on its
own, so one preset covers a document written in either language, or a repository that
carries documents in both.

[日本語](README.ja.md)

## What the rules check

The rules come from the Google developer documentation style guide, from the sections on
lists, on headings, and on timeless documentation. They describe a problem that predates
generative AI: a document that breaks these rules keeps breaking a reader's trust every
time it is edited, whether or not a model wrote it. The exact definition of each rule
lives in [`docs/content/en/metrics.md`](../../docs/content/en/metrics.md) ("The M rules").

The rule key on the left is the name this package registers internally, and it is also the
suffix of the matching Rust rule ID (`maint/list-lead-in` and so on, defined in
`crates/suikou-core/src/rules/mod.rs`). What textlint prints for a finding carries a
different prefix; see "Installing" below.

| Rule key | Checks |
|---|---|
| `list-lead-in` | A list whose lead-in line is grammatically bound to the items, so that adding or removing an item breaks the sentence |
| `item-count` | Prose that states the number of items, so that adding or removing an item makes the prose wrong |
| `numbered-heading` | A heading that carries a hand-written section number |
| `manual-number` | A bulleted list item whose body opens with a hand-written number |
| `parallel-items` | A list whose items do not share one ending shape |
| `time-dependent` | A word whose meaning depends on when the reader opens the document |
| `trailing-etc` | Prose or a list item that trails off with "etc." or "など" |

## Installing

```sh
npm install --save-dev textlint textlint-rule-preset-tech-maintainability
```

Add this package to `.textlintrc`:

```json
{
  "rules": {
    "preset-tech-maintainability": true
  }
}
```

Individual rules can be turned off through this package's namespace:

```json
{
  "rules": {
    "preset-tech-maintainability": {
      "manual-number": false
    }
  }
}
```

A finding loaded this way prints under the prefix `tech-maintainability/`, for instance
`tech-maintainability/list-lead-in`. That prefix comes from the config key above with
`preset-` stripped off; it is textlint's own convention, not something this package
picks. The `maint/` prefix in this document is `suikou check`'s own prefix, kept here
only so the rule key on the right of it (`list-lead-in` and so on) can be matched
against the table above. D-23 in
[`docs/content/en/decisions.md`](../../docs/content/en/decisions.md) has the full
account of why the two prefixes differ.

## Relationship to the Rust implementation

suikou's core analysis lives in a Rust crate
([`crates/suikou-core`](../../crates/suikou-core)) and ships as the `suikou` binary. This
package is a second, independent implementation of the same M rules for people who
already run textlint. The judgment logic and the regular expressions are ported line for
line from `crates/suikou-core/src/rules/mod.rs`; nothing here was invented from scratch.

The two implementations differ in one structural way. `suikou check` builds its blocks
from the raw source, one block per physical line, using the line-based parser in
`markdown.rs`. This package does the same: each rule reads the whole file through
`context.getSource()` and runs it through a JavaScript port of that same line-based
parser, rather than walking textlint's own Markdown AST. A paragraph that soft-wraps
across two lines would otherwise become one joined node inside textlint, and matching
against the joined text can catch a phrase that spans a line break the Rust side never
joins. Porting the line-based parser keeps the count of findings identical between the
two tools; the [golden tests](../../tests/golden/) in this repository check that
identical count file by file.

Japanese morphology is the other point of difference. suikou-core reads one dictionary
through lindera; this package reads a different one, IPADIC, through
[kuromojin](https://github.com/azu/kuromojin), the standard morphological analyzer in
the textlint ecosystem. The first dictionary keeps a suffix, a pronoun, and an
adjectival noun stem apart at the top level; IPADIC does not, and folds all three into
one noun category with a finer subcategory instead. The taigen check used by
`list-lead-in` and `parallel-items` collapses to a single condition under IPADIC as a
result. The reasoning and the probe that confirmed it are written up as D-23 in
[`docs/content/en/decisions.md`](../../docs/content/en/decisions.md).

One more difference is not structural but deliberate: `suikou check` always prints its
explanation in Japanese, even for an English document. This package picks the explanation
language to match the language it detects, because a message nobody in the room can read
does not help them.

## Examples

A lead-in line that ends in a bound predicate breaks the moment the list under it
changes:

```markdown
The files affected are:

- `/etc/app/main.conf`
- `/etc/app/conf.d/*.conf`
```

`list-lead-in` flags the lead-in line above because it ends in a particle, not a full
sentence and not a noun before the colon. Rewriting the lead-in as a full sentence, for
instance "The files affected are as follows:" or "The affected files are these:", fixes
the finding without touching the list itself.

A list whose items do not share one ending shape breaks the same way:

```markdown
- Validate the syntax of the file.
- Reload the service
```

`parallel-items` flags this list because one item ends in a period and the other does
not. Making every item end the same way, either every item as a full sentence or every
item as a phrase, fixes the finding.

## Overlap with other presets

[`@textlint-ja/textlint-rule-preset-ai-writing`](https://github.com/textlint-ja/textlint-rule-preset-ai-writing)
carries a rule named `no-ai-colon-continuation`. It also uses kuromojin to check whether
the text before a colon ends in a predicate, close to the Japanese half of
`list-lead-in`. The purpose differs even where the check overlaps: that rule flags a
phrasing pattern typical of AI-written Japanese, and `list-lead-in` flags a list
structure that breaks when an item is added or removed. Enabling both presets can
produce two findings on the same line. Neither preset suppresses the other; a person who
enabled both can read both messages and judge which one fits their document.

## Testing

```sh
npm test
```

Tests run under Node's built-in test runner (`node --test`), not
[`textlint-tester`](https://github.com/textlint/textlint-tester). `textlint-tester`
assumes Mocha's global `describe` and `it`; its fallback for other runners does not await
the Promise a test case returns, which let assertions pass before the lint they depended
on had finished. `test/support/lint.js` wraps `@textlint/kernel` directly instead, in a
form that awaits cleanly under `node --test`.

`test/golden.test.js` runs this package against every file under
[`tests/golden/input/`](../../tests/golden/input/) and checks that the count of findings,
rule by rule, matches what `suikou check --format json` reports for the same file. That
count is this package's completion bar.

## License

MIT or Apache-2.0, whichever you prefer.
