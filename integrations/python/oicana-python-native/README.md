# oicana-native

> **This is an internal build artifact of [Oicana](https://oicana.com).** Install [`oicana`](https://pypi.org/project/oicana/) instead. It depends on this package and wraps it in the documented API, with type hints, context managers, and enums.

Low-level [PyO3](https://pyo3.rs/) bindings to the Oicana core. The functions here take and return raw handles, with no stability guarantees between releases.

## Development

```bash
uv tool install maturin
maturin develop --release
maturin build --release
```

See [`integrations/python/README.md`](https://github.com/oicana/oicana/blob/main/integrations/python/README.md) for the workspace layout and the test and publish flows.

## Licensing

Oicana is source-available under the [PolyForm Noncommercial License 1.0.0](https://github.com/oicana/oicana/blob/main/LICENSE.md) and free for noncommercial use. Commercial use is free for 30 days; see [pricing](https://oicana.com/#pricing) for subscriptions.
