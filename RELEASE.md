# Creating a Release

[GitHub](https://github.com/orhun/kmon/releases), [crates.io](https://crates.io/crates/kmon/), [AUR](https://aur.archlinux.org/packages/?O=0&SeB=nd&K=Linux+kernel+manager+and+activity&outdated=&SB=n&SO=a&PP=50&do_Search=Go) and [Docker](https://hub.docker.com/r/orhunp/kmon) releases are automated via [GitHub actions](https://github.com/orhun/kmon/blob/master/.github/workflows/cd.yml) and triggered by pushing a tag.

1. Bump the [version](https://semver.org/spec/v2.0.0.html) in [Cargo.toml](https://github.com/orhun/kmon/blob/master/Cargo.toml) and run the app to update [Cargo.lock](https://github.com/orhun/kmon/blob/master/Cargo.lock).
2. Ensure [CHANGELOG.md](https://github.com/orhun/kmon/blob/master/CHANGELOG.md) is updated according to the [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.
3. Commit and push the changes.
4. Create a new tag: `git tag -s -a v[x.y.z]`
5. Push the tag: `git push --tags`
6. Wait for [Continous Deployment](https://github.com/orhun/kmon/actions) workflow to finish.

## License

GNU General Public License ([3.0](https://www.gnu.org/licenses/gpl.txt))