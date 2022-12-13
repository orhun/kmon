# Creating a Release

[GitHub](https://github.com/orhun/kmon/releases), [crates.io](https://crates.io/crates/kmon/) and [Docker](https://hub.docker.com/r/orhunp/kmon) releases are automated via [GitHub actions](https://github.com/orhun/kmon/blob/master/.github/workflows/cd.yml) and triggered by pushing a tag.

1. Bump the version in [Cargo.toml](Cargo.toml) according to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
2. Update [Cargo.lock](Cargo.lock) by building the project: `cargo build`
3. Ensure [CHANGELOG.md](CHANGELOG.md) is updated according to [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.
4. Commit and push the changes.
5. Create a new tag: `git tag -s -a v[x.y.z]` ([signed](https://keyserver.ubuntu.com/pks/lookup?search=0x485B7C52E9EC0DC6&op=vindex))
6. Push the tag: `git push --tags`
7. Wait for [Continuous Deployment](https://github.com/orhun/kmon/actions) workflow to finish.

## License

GNU General Public License ([3.0](https://www.gnu.org/licenses/gpl.txt))
