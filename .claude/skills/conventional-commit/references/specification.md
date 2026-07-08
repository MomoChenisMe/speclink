# Conventional Commits v1.0.0 — Full Specification

Source: https://www.conventionalcommits.org/en/v1.0.0/

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).

## Specification Rules

1. Commits MUST be prefixed with a type, which consists of a noun, `feat`, `fix`, etc., followed by the OPTIONAL scope, OPTIONAL `!`, and REQUIRED terminal colon and space.
2. The type `feat` MUST be used when a commit adds a new feature to your application or library.
3. The type `fix` MUST be used when a commit represents a bug fix for your application.
4. A scope MAY be provided after a type. A scope MUST consist of a noun describing a section of the codebase surrounded by parenthesis, e.g., `fix(parser):`.
5. A description MUST immediately follow the colon and space after the type/scope prefix. The description is a short summary of the code changes, e.g., `fix: array parsing issue when multiple spaces were contained in string`.
6. A longer commit body MAY be provided after the short description, providing additional contextual information about the code changes. The body MUST begin one blank line after the description.
7. A commit body is free-form and MAY consist of any number of newline separated paragraphs.
8. One or more footers MAY be provided one blank line after the body. Each footer MUST consist of a word token, followed by either a `:<space>` or `<space>#` separator, followed by a string value (this is inspired by the [git trailer convention](https://git-scm.com/docs/git-interpret-trailers)).
9. A footer's token MUST use `-` in place of whitespace characters, e.g., `Acked-by` (this helps differentiate the footer section from a multi-paragraph body). An exception is made for `BREAKING CHANGE`, which MAY also be used as a token.
10. A footer's value MAY contain spaces and newlines, and parsing MUST terminate when the next valid footer token/separator pair is observed.
11. Breaking changes MUST be indicated in the type/scope prefix of a commit, or as an entry in the footer.
12. If included as a footer, a breaking change MUST consist of the uppercase text `BREAKING CHANGE`, followed by a colon, space, and description, e.g., `BREAKING CHANGE: environment variables now take precedence over config files`.
13. If included in the type/scope prefix, breaking changes MUST be indicated by a `!` immediately before the `:`. If `!` is used, `BREAKING CHANGE:` MAY be omitted from the footer section, and the commit description SHALL be used to describe the breaking change.
14. Types other than `feat` and `fix` MAY be used in your commit messages, e.g., `docs: updated ref docs`.
15. The units of information that make up Conventional Commits MUST NOT be treated as case sensitive by implementors, with the exception of `BREAKING CHANGE` which MUST be uppercase.
16. `BREAKING-CHANGE` MUST be synonymous with `BREAKING CHANGE`, when used as a token in a footer.

## Commit Message Structure (BNF-style)

```
<commit> ::= <type> ["(" <scope> ")"] ["!"] ":" <space> <description>
             [<blank-line> <body>]
             [<blank-line> <footer>+]

<type>        ::= "feat" | "fix" | "build" | "chore" | "ci"
                | "docs" | "style" | "refactor" | "perf" | "test"
                | <any other noun>

<scope>       ::= <noun>          ; e.g., parser, auth, api, lang

<description> ::= <text>          ; short summary, imperative mood

<body>        ::= <text>          ; free-form, may contain multiple paragraphs

<footer>      ::= <token> <separator> <value>
<token>       ::= "BREAKING CHANGE" | "BREAKING-CHANGE" | <word> ["-" <word>]*
<separator>   ::= ": " | " #"
<value>       ::= <text>          ; may span multiple lines
```

## SemVer Correlation

Conventional Commits maps directly to [Semantic Versioning](https://semver.org/):

| Commit Content | SemVer Impact | Version Bump |
|---|---|---|
| `fix` type | Backward compatible bug fix | PATCH (x.y.**Z**) |
| `feat` type | Backward compatible new feature | MINOR (x.**Y**.z) |
| `BREAKING CHANGE` footer or `!` after type/scope | Incompatible API change | MAJOR (**X**.y.z) |

- A commit that has a footer with `BREAKING CHANGE:`, or appends a `!` after the type/scope, introduces a breaking API change (correlating with MAJOR in SemVer).
- A `feat` commit correlates with a MINOR version bump.
- A `fix` commit correlates with a PATCH version bump.
- Other types (build, chore, ci, docs, style, refactor, perf, test) do NOT directly correlate with a SemVer bump, but MAY still be included in changelogs.

## FAQ (Selected)

**Q: How should I deal with commit messages in the initial development phase?**
A: Proceed as if you've already released the product. Typically *somebody*, even if it's your fellow software developers, is using your software. They'll want to know what's fixed, what breaks, etc.

**Q: Are the types in the commit title uppercase or lowercase?**
A: Any casing may be used, but it's best to be consistent. Lowercase is the most common convention.

**Q: What do I do if the commit conforms to more than one of the commit types?**
A: Go back and make multiple commits whenever possible. Part of the benefit of Conventional Commits is its ability to drive us to make more organized commits and PRs.

**Q: Doesn't this discourage rapid development and fast iteration?**
A: It discourages moving fast in a disorganized way. It helps you be able to move fast long term across multiple projects with varied contributors.

**Q: How does this relate to SemVer?**
A: `fix` type commits should be translated to PATCH releases. `feat` type commits should be translated to MINOR releases. Commits with `BREAKING CHANGE` in the commits, regardless of type, should be translated to MAJOR releases.

**Q: What do I do if I accidentally use the wrong commit type?**
A: If you used a type that is of the spec but not the correct type (e.g., `fix` instead of `feat`), prior to merging or releasing the mistake, use `git rebase -i` to edit the commit history. After release, the cleanup will be different according to what tools and processes you use.

**Q: Do all my contributors need to use the Conventional Commits specification?**
A: No! If you use a squash-based workflow on Git, lead maintainers can clean up the commit messages as they're merged — adding no workload to casual committers. A common workflow for this is to have your git system automatically squash commits from a pull request and present a form for the lead maintainer to enter the proper git commit message for the merge.
