# Pact WASM bindings

Provides a WASM binary that includes the Pact models.

## Building for use on websites

The project is built using `wasm-pack`. To build a package that can be used on a website:

```commandline
$ wasm-pack build -t web --release
```

This will create a pkg directory with all the files needed to load from a webpage.
