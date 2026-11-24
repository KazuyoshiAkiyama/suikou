# suikou

suikou is a linter and harness for technical documentation in Japanese and English.

Technical documentation produced by generative AI shows patterns that go further than
vocabulary. The text uses many figures of speech, keeps ending sentences the same way,
connects sentences clumsily, and includes detail that makes the document hard to maintain
later. suikou detects these patterns in a document.
It then reshapes the document so that it remains easy to maintain.

[日本語](README.ja.md) · [Documentation](docs/)

## How suikou works

### Each rule is based on a measurement or a published standard

No rule in suikou was chosen on intuition.
When intuitions about style are measured, most of them prove wrong.
Research on translationese and research on AI-written text reached this conclusion
independently of each other.

`corpus/baselines.toml` holds the measured values behind each threshold.
It also lists the metrics that were rejected, along with the reason for each rejection.
Keeping the rejected metrics on record means that no one has to test the same intuition again.

### Findings fall into two groups

Some problems can be judged true or false within a single line or sentence, and these
problems have a position in the document. Other problems can be measured only over the
whole document, and these problems have no position. Most problems in the second group
point out something the document lacks, and a missing element has no location to report.

This division is what allows the work to finish with a single run and a single round of
edits. Findings with a position tell the writer where to edit.
Guidance derived from the whole document tells the writer how to edit.
Since both are delivered at once, no layer needs a separate pass of linting and fixing.

### The maintainability layer is independent of AI style

Rules M1 through M7 are taken from the Google developer documentation style guide.
These rules address a problem that existed before generative AI, so they stay valid as
models change. For this reason, the layer can also be used by itself. A document written
before generative AI existed, for example, violates these rules in the same way.

## Installation

GitHub Releases provides an archive for each operating system.
The dictionary is built into every archive, so nothing more needs to be downloaded.
The Linux build is statically linked against musl, so it runs regardless of the glibc version.

```sh
tar xf suikou-v<version>-<target>.tar.xz
cd suikou-v<version>-<target>
install -m 755 suikou ~/.local/bin/
suikou selftest
```

On Windows, extract the zip file and place `suikou.exe` in a directory on the PATH.

`suikou selftest` loads the dictionary and has it analyze a sentence.
If your binary lacks the dictionary, the command fails at that point.
The check is designed this way because a printed flag can report a dictionary that was
never linked into the binary.

To install from source, specify the feature.

```sh
cargo install --git https://github.com/KazuyoshiAkiyama/suikou suikou-cli --features lindera-unidic
```

Without the feature, the installed build contains no dictionary.
That build supports English only, and it fails immediately when asked to process Japanese.
If it returned an empty analysis instead, every Japanese metric would become zero.
Zero is below the thresholds, so the run would falsely report that nothing needs fixing.

## Usage

Start a document by choosing its type. The type is settled in two steps at most.

```sh
suikou doctype
```

Next, write the outline before the prose.

```sh
suikou plot docs/guide.md --doctype howto
```

This command lists, under `.suikou/plans/`, the heading of each section and the question
that section answers. Write the claim of each section in one line, and have the claims
approved before you write the body. The plot stays out of version control, because it
records the thinking and is not a deliverable.

Pass the doctype when you build the instruction for a model as well. The instruction then
lists the question for each section and the writing rules before the prohibitions.

```sh
suikou brief --doctype howto --lang en
```

Check the written document afterward.

```sh
suikou check docs/guide.md
```

`suikou mcp` launches an MCP server that communicates with JSON-RPC 2.0 over stdio and
responds to `initialize`, `tools/list`, and `tools/call`. The server offers `check` and
`brief` as tools. An agent that supports MCP can therefore invoke the same judgment as the
CLI without parsing output meant for a terminal.

```sh
suikou mcp
```

To register the server, set `suikou` as the command and `mcp` as its argument in the
MCP client's server configuration.

See the [documentation](docs/) for complete installation and usage instructions.

## Documentation

The documentation is in `docs/`. It is built with Hugo and is available in both English
and Japanese, with English as the default language.

```sh
cd docs && hugo server
```

| Page | Content |
|---|---|
| design | The overall design, including the measurements that support it |
| metrics | The exact definition of each metric, for anyone porting them |
| decisions | The reasoning behind each decision |
| writing | Norms for word choice and README structure |
| development | Repository layout, build steps, version pinning, and tokenizer handling |

`CLAUDE.md` lists the constraints, and `TASKS.md` lists the order of work.

## Status

All subcommands are implemented. The exception is `daemon`, which was rejected: the
binary starts in a few milliseconds, so a resident process would save nothing. Tests,
fmt and clippy pass both with the default features and with `--features lindera-unidic`.
The golden tests keep the Rust rules, the textlint rules and the reference implementation
returning the same counts.

The status of each part is kept in `CLAUDE.md` instead of here, because a second copy
falls out of date. The copy in this README was five parts behind by the time it was removed.

## License

You may use suikou under either the MIT or the Apache-2.0 license.
