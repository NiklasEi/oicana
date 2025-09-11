# Contributing to Oicana

First of all: thank you for your interest!

For the moment, please only open GitHub issues. A CLA process to allow for code contributions is work in progress.

## Project structure

Oicana consist of Rust libraries around the Typst compiler and multiple integrations for different tech stacks.

### Top-level directories

- `assets` - test files and logos
- `crates` - Rust libraries wrapping the Typst compiler and implementing other features like snapshot testing
- `docs` - documentation source files; using Typst of course ;)
- `e2e_test_template` - Oicana template that is used for e2e testing
- `integrations` - projects integrating Oicana into different tech stacks
- `tools` - Oicana CLI and a tool for this repository's e2e tests
