# Contributing

This document outlines how to propose a change to the Axel project.
If you have any further questions please create an issue or ask in
the [KonoSuba: Fantastic Days Discord server](https://discord.gg/3gFQMgE).

## Code style

New code should follow the existing style of the project, defined in [rustfmt.toml](../rustfmt.toml)
(main rule is 2 space indents). You can run `cargo +nightly fmt` to automatically format your code.
Please don’t restyle code that has nothing to do with your PR.

If your code looks better without rustfmt (see examples in codebase), you can disable it with
`#[rustfmt::skip]` or `#[cfg_attr(rustfmt, rustfmt::skip)]`.

## Code changes

Address a single concern with the least number of changes possible.
This makes it easier to review changes and find bugs later.

- Do not edit [masters](../master), i.e. client configuration files.
  If needed, modify them dynamically in [`patch_master` function](../src/api/master_all.rs).
- Do not edit existing [migrations](../migrations), create new ones instead.
- Do not add new dependencies without necessity.
- Please, do not submit "cleanup" PRs that reformat code or remove unused code or files, I can do it myself when needed.

### Bigger changes

If you want to make a bigger change, it’s a good idea to first ask in Discord server
or file an issue and make sure someone from the team agrees that it’s needed.

## Continuous Integration

[Main server](https://axel.assasans.dev/static/) runs on the main branch,
so make sure your changes always keep it buildable and functional.

## Licensing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or https://apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

without any additional terms or conditions.
