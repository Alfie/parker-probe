#!/usr/bin/env node

const { Binary } = require("binary-install");

//TODO: change name and link to  wasm you have control of on github
//      change name in package.json bin as well 
let binary = new Binary('my-binary', 'https://example.com/binary/tar.gz', 'v1.0.0')
binary.run();