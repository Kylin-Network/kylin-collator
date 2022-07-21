# Contributing to predicates-rs

Thanks for wanting to contribute! There are many ways to contribute and we
appreciate any level you're willing to do.

## Feature Requests

Need some new functionality to help?  You can let us know by opening an
[issue][new issue]. It's helpful to look through [all issues][all issues] in
case its already being talked about.

## Bug Reports

Please let us know about what problems you run into, whether in behavior or
ergonomics of API.  You can do this by opening an [issue][new issue]. It's
helpful to look through [all issues][all issues] in case its already being
talked about.

## Pull Requests

Looking for an idea? Check our [issues][issues]. If it's look more open ended,
it is probably best to post on the issue how you are thinking of resolving the
issue so you can get feedback early in the process. We want you to be
successful and it can be discouraging to find out a lot of re-work is needed.

Already have an idea?  It might be good to first [create an issue][new issue]
to propose it so we can make sure we are aligned and lower the risk of having
to re-work some of it and the discouragement that goes along with that.

### Process

When you first post a PR, we request that the the commit history get cleaned
up.  We recommend avoiding this during the PR to make it easier to review how
feedback was handled. Once the commit is ready, we'll ask you to clean up the
commit history.  Once you let us know this is done, we can move forward with
merging!  If you are uncomfortable with these parts of git, let us know and we
can help.

We ask that all new files have the copyright header.  Please update the
copyright year for files you are modifying.

For commit messages, we use [Conventional](https://www.conventionalcommits.org)
style.  If you already wrote your commits and don't feel comfortable changing
them, don't worry and go ahead and create your PR.  We'll work with you on the
best route forward. You can check your branch locally with
[`committed`](https://github.com/crate-ci/committed).

As a heads up, we'll be running your PR through the following gauntlet:
- warnings turned to compile errors
- `cargo test`
- `rustfmt`
- `clippy`
- `rustdoc`
- [`committed`](https://github.com/crate-ci/committed)

## Releasing


When we're ready to release, a project owner should do the following
- Run `cargo release --push-remote upstream --dry-run -vv patch` to verify changes
- Run `cargo release --push-remote upstream patch` to apply changes

[issues]: https://github.com/assert-rs/predicates-rs/issues
[new issue]: https://github.com/assert-rs/predicates-rs/issues/new
[all issues]: https://github.com/assert-rs/predicates-rs/issues?utf8=%E2%9C%93&q=is%3Aissue
[travis]: https://github.com/assert-rs/predicates-rs/blob/master/.travis.yml
