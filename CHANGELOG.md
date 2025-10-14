# Next release

# Crates v0.1.0-alpha.4 and CLI v0.1.0-alpha.5

- fix fuzzing tests from paths other than template root ((#28)[https://github.com/oicana/oicana/pull/20])

# Crates v0.1.0-alpha.3 and CLI v0.1.0-alpha.4

- mostly changes for CLI - v0.1.0-alpha.3

# CLI - v0.1.0-alpha.3

- fix CLI sometimes packaging test files ((#20)[https://github.com/oicana/oicana/pull/20])
- tests can be without a snapshot file ((#16)[https://github.com/oicana/oicana/pull/16])
- tests can fuzz json inputs with a json schema ((#16)[https://github.com/oicana/oicana/pull/16])
- tests will fail for missing snapshots ((#17)[https://github.com/oicana/oicana/pull/17])
- new options for test command `--update`/`-u` will overwrite/create snapshot files ((#17)[https://github.com/oicana/oicana/pull/17])
