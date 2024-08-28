# Project process

This document describes the process of creating this project and its challenges.

## Overview

The process of writing this project was somewhat time consuming, but rewarding. Around midway in the project, I started trialing (Cursor)[https://www.cursor.com/] for code completion and correction. This proved to be rather useful.

Overall, this project went quite well. There are a lot of things that could be still added, but this was never meant to be fully production ready. Biggest challenges were related to components I was not familiar with, for example Shuttle for deployment.

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
- Add state storage to the server and recovery from a restart
- Implement the possibility of submitting files multiple times at the client side
- Continue with better documentation
- Add better error handling

## Main docs

Overview of the project, installation, deployment and usage instructions can be found in the [README](README.md) file.