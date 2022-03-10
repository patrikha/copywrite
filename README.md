# copywrite

Utility to add or update copyright header in source files. Uses [tera](https://crates.io/crates/tera) templates as basis for banner definitions.
All environment variables are available in the template specification as well as the special {{year}} that will resolve to the current year.

Sample of a template:
```
Copyright © {{year}} Acme Corporation
```

## Supported languages
* C/C++
* C#
* Python
* Rust
* Go
* Swift
* Objective-C
* Kotlin
* Java
* JavaScript
* Groovy
* PHP
* TypeScript
* HTML
* CSS
* SVG
* XML

## Usage:
```
copywrite

Usage:
  copywrite <path> --template=TEMPLATE [--exclude=PATH]... [--gitindex]
  copywrite (-h | --help)
  copywrite --version
  copywrite -v

Options:
  --template=TEMPLATE  Path to tera (Jinja2) template file containing the copyright banner. All environment variables plus {{year}}
                       for current year are available in the template.
  --exclude=PATH       Exclude path, file och directory, can be repeated multiple times
  --gitindex           Filter on files in git index only
  -h --help            Show this screen
  --version            Show version
  -v                   Show shorthand version string
```

## Building from source
copywrite is built using the Rust language and to get started just install the Rust tool-chain:
[Install](https://www.rust-lang.org/tools/install)

Then pull down the dependencies and build the tool:

``` bash
cargo build
```
