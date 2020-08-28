# MONEY

Cash management **crates** for *Rust* and *Web* applications.

## Project structure

### Rust crates

First of all, there are 2 *Rust Workspaces*:

* `rust-money`: core **API** implementation for *Rust* applications.
* `wasm-money`: `money` for *WebAssembly* target and *JavaScript/TypeScript* binding.

### Web application

There is also the `webpack-app` project which uses `wasm-money` artifacts to provide a web-based front-end solution.

> This structure is a temporary one as `rust-money` is not published yet, it enables developping and testing things easily.

## MacOS setup

Follow these steps:

```zsh
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install "wasm-pack"
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install Node
brew install node
```

## Build and test Rust crates

Make sure that tools are correctly installed by building local *Rust* **crates**:

```zsh
# Build each workspace binary for host target
cargo build
```

Run each test using **host** target:

```zsh
# Test workspaces (default target)
cargo test
```

> Running tests automatically triggers the corresponding build before.

## Build WebAssembly binary and JavaScript interface

Then attempt to cross-compile the *WebAssembly* binary - from `wasm-money` sources - and generate both *JavaScript* and *TypeScript* interfaces
using this command:

```zsh
# Build 'wasm-money' workspace for WebAssembly target
# Generate binding code too
wasm-pack build wasm-money
```

> This command automatically calls `cargo build` for `wasm-money`only, but for target `wasm32-unknown-unknown`!
> Both binary and binding code will be generated into folder `wasm-money/pkg`.

## Run the web application

We can now initialize the web server of the *webpack* application:

```zsh
cd webpack-app

# Initialize the web server
npm install
```

> Must be performed once!

The web server is now ready to use:

```zsh
# Start the web server
npm run start
```

> It is more convenient to run this command in a seperate terminal.

After that, you should be able to visit `localhost:8080` from the web browser of your choice!

## Test the web application

Run tests which are specific to target `wasm32-unknown-unknown` using one of these commands according to your web browser:

```zsh
# Run specific tests within web browser (WebAssembly target only)
wasm-pack test wasm-money --safari --headless
wasm-pack test wasm-money --chrome --headless
wasm-pack test wasm-money --firefox --headless
```

> Make sure to refresh your binary first with `wasm-pack build`, it seems not automatic this time...

## Edit sources

As the *Node* server automatically fetches the folder `wasm-money/pkg`, you just need to [build the WebAssembly binary](#build-webAssembly-binary-and-javascript-interface) to refresh the web page once you edited sources.

Enjoy! 🤠
