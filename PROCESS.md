# Project process

This document describes the process of creating this project and its challenges.

## Overview

The process of writing this project was somewhat time consuming, but rewarding. Around midway in the project, I started trialing (Cursor)[https://www.cursor.com/] for code completion and correction. This proved to be rather useful.

Overall, this project went quite well. There are a lot of things that could be still added, but this was never meant to be fully production ready. Biggest challenges were related to components I was not familiar with, for example Shuttle for deployment. Also getting the Merkle Tree to work correctly was a hassle - unit tests helped there.

## Missing features

Future features that should be added:
- Add more unit tests
- Implement security features:
  - Authentication
  - Authorization
  - Throttling
  - Rate limiting
  - Audit logs
  - ...
- Wrap the server in a Docker container
- Generate proper API documentation for the server
- Add state storage to the server and recovery from a restart
- Implement the possibility of submitting files multiple times at the client side
- Continue with better documentation
- Add better error handling

## Very short usage instructions

1. Generate dummy files: `./generate.sh`
1. Upload all files: `cargo run --bin client -- upload https://merkleproofs.shuttleapp.rs all`
1. Verify a specific file: `cargo run --bin client -- verify https://merkleproofs.shuttleapp.rs 1`

More thorough instructions can be found in the [README](README.md) file.