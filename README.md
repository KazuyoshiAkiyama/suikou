# suikou

A linter and harness for technical documentation. It handles Japanese and English.

Technical documentation written by generative AI carries habits that go beyond word choice.
It leans on figures of speech, it repeats the same sentence endings, it joins sentences
awkwardly, and it spells out details that make the document hard to maintain later.
suikou finds these habits in the document.
It then moves the document toward a shape that stays maintainable.

[日本語](README.ja.md) · [Documentation](docs/)

## How it is built

### Every rule rests on a measurement or on a written standard

Not one rule here came from a hunch.
Hunches about style turn out to be wrong most of the time once someone measures them.
Research on translationese and research on AI prose found this same result, and the two
lines of research were separate.

The measurements behind each threshold live in `corpus/baselines.toml`.
The metrics that were rejected are recorded there as well, together with the reason.
They stay on record so that nobody has to test the same hunch twice.

### What the tool finds splits into two groups

Some problems are true or false at the level of a line or a sentence, and those problems
carry a position. Other problems can only be measured across the whole document, and
those problems carry no position. Most of the problems in the second group report
something that is missing from the document, and something that is missing has nowhere
to point.

That split is what lets one run and one edit finish the work.
The findings that carry a position say where to edit.
The guidance built from the whole document says how to edit.
Because both arrive together, no layer has to be linted and fixed in a separate pass.

### The maintainability layer stands apart from AI style

Rules M1 through M7 come from the Google developer documentation style guide.
Those rules describe a problem that predates generative AI, so those rules do not age as
models change. That is why this layer can be used on its own. A document written before
generative AI existed, for example, breaks the same rules in the same way.

## Installing

GitHub Releases carries an archive for each operating system.
Each archive embeds the dictionary, so nothing has to be downloaded afterward.
The Linux build links musl statically, so it runs whatever version of glibc is present.

```sh
tar xf suikou-v<version>-<target>.tar.xz
cd suikou-v<version>-<target>
install -m 755 suikou ~/.local/bin/
suikou selftest
```

On Windows, open the zip and put `suikou.exe` somewhere on the PATH.

`suikou selftest` loads the dictionary and asks the dictionary to analyze a sentence.
If the binary you hold is missing the dictionary, the command fails there.
The check works this way because a printed flag can claim a dictionary that was never
linked into the binary.

To install from source, name the feature.

```sh
cargo install --git https://github.com/KazuyoshiAkiyama/suikou suikou-cli --features lindera-unidic
```

Leaving the feature out installs a build with no dictionary.
That build handles English only. Asking that build for Japanese makes it fail on the spot.
Returning an empty analysis would drive every Japanese metric to zero.
Zero sits below the thresholds, so the run would end in a false report of nothing to fix.

## Using it

```sh
suikou check docs/guide.md
```

`suikou mcp` starts an MCP server that speaks JSON-RPC 2.0 over stdio and answers
`initialize`, `tools/list`, and `tools/call`. It exposes `check` and `brief` as tools,
so an agent that supports MCP calls the same judgment the CLI uses, without parsing
text meant for a terminal.

```sh
suikou mcp
```

Point an MCP client at that command by naming `suikou` and `mcp` as the command and
its argument in the client's server configuration.

The [documentation](docs/) covers installation and use in full.

## Documentation

The documentation lives in `docs/`. The documentation builds with Hugo, and it carries
both English and Japanese. English is the default language.

```sh
cd docs && hugo server
```

| Page | Content |
|---|---|
| design | The design as a whole, with the measurements behind it |
| metrics | The strict definition of each metric, for anyone porting them |
| decisions | The reasoning behind each decision |
| development | How to build, how versions are pinned, how the tokenizer is handled |

`CLAUDE.md` records the constraints, and `TASKS.md` records the order of work.

## Layout

| Path | Content |
|---|---|
| `crates/suikou-core` | The analysis core |
| `crates/suikou-cli` | The CLI and the MCP server |
| `packages/` | The textlint rules that carry a position |
| `skills/suikou` | The skill for agents |
| `research/` | The Python used for calibration and exploration |
| `corpus/` | The measurements and the script that fetches the corpus |

## State

Every subcommand is written, except that `daemon` was decided against. Launching the
binary takes a few milliseconds, which leaves a resident process nothing to save.
Tests, fmt, and clippy pass under the default features and under
`--features lindera-unidic`.

| Part | State |
|---|---|
| Block extraction and preprocessing | Written, with unit tests |
| Rules M1 through M7 | Written, with unit tests |
| Japanese and English document metrics | Written |
| The `check` subcommand | Written |
| The lindera binding | Written and checked against lindera 6.0.0 |
| Golden tests | Wired up for the rules and the document metrics |
| `brief`, `baseline`, `terms`, and `mcp` | Written |
| The textlint rules | Written |

## License

MIT or Apache-2.0, whichever you prefer.
