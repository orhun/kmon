# Contributing

Thank you for considering to contribute to [kmon](https://github.com/orhun/kmon/)!

When contributing, please first discuss the change you wish to make via [issue](https://github.com/orhun/kmon/issues),
[email](mailto:orhunparmaksiz@gmail.com), or any other method with the owners of this repository before making a change.

Please note we have a [code of conduct](https://github.com/orhun/kmon/blob/master/.github/CODE_OF_CONDUCT.md), please follow it in all your interactions with the project.

## Setup

1. Fork this repository and create your branch from `master`.
```
git clone https://github.com/[username]/kmon && cd kmon
```

2. Build the project for installing the dependencies.
```
cargo build
```

3. Use `cargo run` command for starting the terminal interface while development.

4. Add your tests or update the existing tests according to the changes.
```
cargo test --all
```

5. Make sure [rustfmt](https://github.com/rust-lang/rustfmt) and [clippy](https://github.com/rust-lang/rust-clippy) pass before creating a pull request.
```
cargo fmt --all -- --check
cargo clippy -- -D warnings
```

## Create a Pull Request

1. Ensure any install or build dependencies are removed before the end of the layer when doing a build.

2. Update the [README.md](https://github.com/orhun/kmon/blob/master/README.md) with details of changes to the terminal user interface including new environment variables, command line arguments and container parameters.

3. Increase the version number in [Cargo.toml](https://github.com/orhun/kmon/blob/master/Cargo.toml) to the new version that this Pull Request would represent. The versioning scheme we use is [SemVer](http://semver.org/).

4. You may merge the Pull Request in once you have the sign-off of the two other developers, or if you do not have permission to do that, you may request the second reviewer to merge it for you.

# License 
By contributing, you agree that your contributions will be licensed under [GNU General Public License 3.0](https://github.com/orhun/kmon/blob/master/LICENSE).