# suikou

suikou is a linter and authoring harness for Japanese and English technical documentation.

Technical documentation produced by generative AI exhibits recurring issues beyond vocabulary selection: frequent figurative language, repetitive sentence endings, awkward clause transitions, and excessive detail that complicates maintenance. suikou detects these patterns and helps revise the text into maintainable documentation.

[日本語](README.ja.md) · [Documentation](docs/)

## How it works

### Rules backed by measurements or published standards

Every rule in suikou derives from empirical measurement or a published style standard rather than intuition. Quantitative evaluation disproves most intuitive assumptions about writing style. Translationese research and AI-prose research reached this same conclusion independently across two separate lines of inquiry.

Threshold measurements are stored in `corpus/baselines.toml`. Evaluated metrics that were rejected are documented in the same file along with the rationale for excluding them, preventing anyone from testing the same hypothesis twice.

### Positional diagnostics and document-wide guidance

Line- and sentence-level checks evaluate binary conditions and report an exact source position. Document-wide checks measure properties across the full file without a source position, because most of them identify omitted elements that have no specific location.

This separation enables resolving all findings in one run and one edit. Positional findings specify where to make changes, while document-wide guidance specifies how to revise the text. Producing both outputs together removes the need for separate lint-and-fix passes per layer.

### Maintainability rules independent of AI style

Rules M1 through M7 are derived from the Google developer documentation style guide. Because these rules address documentation flaws that predate generative AI, they remain valid regardless of how language models evolve. This independence allows the maintainability layer to be used standalone; documentation authored before generative AI existed violates the same rules in the same ways.

## Install

Prebuilt archives for each operating system are published on GitHub Releases. Every archive bundles the dictionary so no post-install downloads are needed, and the Linux binary links musl statically to run independently of the host glibc version.

```sh
tar xf suikou-v<version>-<target>.tar.xz
cd suikou-v<version>-<target>
install -m 755 suikou ~/.local/bin/
suikou selftest
```

On Windows, extract the zip archive and place `suikou.exe` in a directory listed on PATH.

`suikou selftest` loads the dictionary and runs analysis on a test sentence, failing immediately if the binary lacks dictionary data. Performing an actual analysis prevents false confirmation from a static flag claiming dictionary support that was never linked into the executable.

To build and install from source, enable the dictionary feature explicitly.

```sh
cargo install --git https://github.com/KazuyoshiAkiyama/suikou suikou-cli --features lindera-unidic
```

Omitting this feature compiles a build without a dictionary that supports English only and exits with an error on Japanese input. Returning an empty analysis instead would evaluate every Japanese metric to zero, remaining below every threshold and falsely reporting no issues.

## Usage

Start a new document by determining its document type. Interactive prompts identify the classification in at most two steps.

```sh
suikou doctype
```

Next, create a structural outline before drafting prose.

```sh
suikou plot docs/guide.md --doctype howto
```

This command generates the section headings and the guiding question for each section under `.suikou/plans/`. Write a one-line claim for each section and confirm approval before drafting the body text. Exclude these plan files from version control because they record intermediate planning rather than a final deliverable.

Generate model instructions for the selected document type so that each section's guiding question and affirmative writing rules appear before any prohibitions.

```sh
suikou brief --doctype howto --lang en
```

Run checks against the drafted document.

```sh
suikou check docs/guide.md
```

`suikou mcp` launches an MCP server communicating via JSON-RPC 2.0 over stdio and supporting `initialize`, `tools/list`, and `tools/call`. By exposing `check` and `brief` as tools, it enables MCP-compatible agents to invoke the same evaluation logic as the CLI without parsing terminal output.

```sh
suikou mcp
```

Configure an MCP client to connect by specifying `suikou` as the command and `mcp` as its argument in the client's server configuration.

See the [documentation](docs/) for complete installation and usage instructions.

## Documentation

Project documentation is stored in `docs/`, built with Hugo, and provided in both English and Japanese with English as the default language.

```sh
cd docs && hugo server
```

| Page | Overview |
|---|---|
| design | Overall architecture and the empirical measurements supporting it |
| metrics | Formal definitions of every metric for porting to other implementations |
| decisions | Rationale for each design decision |
| writing | Conventions for word choice and README layout |
| development | Repository structure, build procedures, version pinning, and tokenizer handling |

Project constraints are documented in `CLAUDE.md`, and the sequence of work is tracked in `TASKS.md`.

## Status

All planned subcommands are implemented except `daemon`, which was rejected because startup takes only a few milliseconds and a resident process would yield no meaningful performance gain. The test suite, fmt, and clippy pass under both the default features and `--features lindera-unidic`. Golden tests enforce identical counts across the Rust rules, the textlint rules, and the reference implementation.

Component-level status is tracked in `CLAUDE.md` rather than duplicated here, because a second copy diverges from the source of truth; an earlier copy in this file was behind by five parts before it was removed.

## License

Dual-licensed under MIT or Apache-2.0, at your option.
