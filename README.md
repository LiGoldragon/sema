# sema

The sema database — content-addressed record storage for typed
program structure. Pseudo-sema while the system bootstraps: records
are rkyv-archived Rust values from
[signal](https://github.com/LiGoldragon/signal), stored in
[redb](https://github.com/cberner/redb), addressed by their slot.
Content-addressing by BLAKE3 hash lands as kinds beyond Node /
Edge / Graph come online.

A `Graph` record is the database-level compilation unit — a
flow-graph of `Node` records connected by `Edge` records that
compile together to one artifact. The flow-graph IS the
program.

## License

[License of Non-Authority](LICENSE.md).

## Reference

[reference/Vision.md](reference/Vision.md) — aspirational description
of the mature sema format (universal typed-binary of meaning,
self-transforming via quorum-signed spec changes, criome as the
distributed web). Reference material for future direction; current
implementation is the pseudo-sema layer above.
