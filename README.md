## Build
```bash
# Install rustup
curl https://sh.rustup.rs -sSf | sh
# Set env
source "$HOME/.cargo/env"

# verify install
rustc -V

# Build
cargo build --release

# Run
cargo run

``` 

## Install
install `gh` from [github](https://cli.github.com/)
login into your account `gh auth login`
install extension `gh extension install mislav/gh-repo-collab`

## Usage
Download github-migration from releases and install it into your path

`github-migrator --help`

See example for migration script