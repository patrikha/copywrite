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

Add or update copyright banner in source files.

USAGE:
    copywrite.exe [OPTIONS] --template <TEMPLATE> <PATH>

ARGS:
    <PATH>    Path to update with copyright template.

OPTIONS:
    -d, --gitstaged              Filter on files added to git staging index only.
    -e, --exclude <EXCLUDE>      Exclude path, file or directory name, can be repeated.
    -g, --gitindex               Filter on files in git index only.
    -h, --help                   Print help information
    -l, --language <LANGUAGE>    Restrict to only update files for specified language(s), can be
                                 repeated.
    -t, --template <TEMPLATE>    Path to tera (Jinja2) template file containing the copyright
                                 banner. All environment variables plus {{year}} for current year
                                 are available in the template.
    -v                           Prints shorthand version information.
    -V, --version                Print version information
```

## Building from source
copywrite is built using the Rust language and to get started just install the Rust tool-chain:
[Install](https://www.rust-lang.org/tools/install)

Then pull down the dependencies and build the tool (you will also need perl on your system):

``` bash
cargo build
```
