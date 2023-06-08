#!/usr/bin/env node

const { Binary } = require("binary-install");

//TODO: - store the binary on github and fetch it from there
//      - to add pkg and commit wasm to github so it can be accessed
let binary = new Binary('my-binary', 'https://example.com/binary/tar.gz', 'v1.0.0')
binary.install();