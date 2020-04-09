# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0] - 2020-04-09
### Added
- Add `-d, --dependent` flag for sorting modules by their dependent modules
- Add information about `-d, --dependent` flag to README.md
- Add a section to README.md about installation from nixpkgs

### Fixed
- Fix the CI workflow about Docker builds

## Changed
- Update README.md about sorting/reversing GIFs
- Improve the test cases of sort type flags

## [1.0.1] - 2020-04-05
### Added
- Add Copr package instructions to README.md

### Fixed
- Fix the broken manpage link in README.md

## [1.0.0] - 2020-04-01
### Added
- Add roadmap, ToC, funding information and new images to README.md
- Add FUNDING.yml

## [0.3.3] - 2020-03-23
### Fixed
- Update the AUR (git) release step in CD workflow

## [0.3.2] - 2020-03-23
### Added
- Add snapcraft.yml for the snap package

### Fixed
- Fix the AUR publish actions in CD workflow according to the package guidelines

### Changed
- Update .gitignore and .dockerignore files about snap package files
- Update README.md about the main usage gif

## [0.3.1] - 2020-03-19
### Fixed
- Fix stylize function about adding colors to the text

### Changed
- Update README.md about usage information, features and images
- Update descriptions of the module management commands
- Update the module blacklisting command

## [0.3.0] - 2020-03-10
### Added
- Add horizontal scrolling feature to kernel activities block
- Add debug derive to enum types

### Fixed
- Use the case insensitive alt-key combinations

### Changed
- Update the runtime key bindings
- Update README.md about key bindings and sections
- Update the manual page about key bindings

## [0.2.2] - 2020-03-01
### Added
- Update README.md about man page and project description

### Changed
- Move man page file to man/ directory
- Update the CD workflow about the new location of man page

## [0.2.1] - 2020-02-28
### Added
- Add project installation, usage, key bindings and resources to README
- Add manual page for the project
- Add test for the `ctrl-l` key action

### Changed
- Update the CD workflow for adding the manual page to the final binary package

## [0.2.0] - 2020-02-23
### Added
- Add key bindings for clearing the kernel ring buffer
- Add `--ctime` parameter to `dmesg` command for human readable date format

## [0.1.1] - 2020-02-23
### Added
- Add contribution guidelines, release instructions and changelog

### Fixed
- Improve the CI/CD workflows

### Changed
- Update the documentation

## [0.1.0] - 2020-02-06
### Added

- Add CI/CD workflows to the project for automation
