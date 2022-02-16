# rbspy

This is a fork of [rbspy — Sampling profiler for Ruby](https://github.com/rbspy/rbspy) that turns it into a C library used in [🔥 Pyroscope](https://github.com/pyroscope-io/pyroscope). All the credit goes to Julia Evans and others behind [rbspy project](https://github.com/rbspy).
### Standalone binary


### As a Rust library

To use rbspy in your Rust project, add the following to your Cargo.toml:

```toml
[dependencies]
rbspy = "0.8"
```

**WARNING**: The rbspy crate's API is not stable yet. We will follow [semantic versioning](https://semver.org/) after rbspy reaches version 1.0.
