# sema

The sema database — content-addressed record storage for typed
program structure. Pseudo-sema while the system bootstraps: records
are rkyv-archived Rust values from
[nexus-schema](https://github.com/LiGoldragon/nexus-schema), stored
in [redb](https://github.com/cberner/redb), addressed by their
blake3 hash.

An opus is a database-level compilation unit — a collection of
records rooted at a module that compile together to one artifact.

## License

[License of Non-Authority](LICENSE.md).

## Reference

[reference/Vision.md](reference/Vision.md) — aspirational description
of the mature sema format (universal typed-binary of meaning,
self-transforming via quorum-signed spec changes, criome as the
distributed web). Reference material for future direction; current
implementation is the pseudo-sema layer above.
