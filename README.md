# sema

The sema database — content-addressed record storage for typed
program structure. Pseudo-sema while the system bootstraps: records
are rkyv-archived Rust values from
signal, stored in
[redb](https://github.com/cberner/redb), addressed by their slot.
Content-addressing by BLAKE3 hash lands as kinds beyond Node /
Edge / Graph come online.

The typed-kernel surface stores a Sema database header alongside
the schema version, so each database records the rkyv format
identity it was written with.

A `Graph` record is the database-level compilation unit — a
flow-graph of `Node` records connected by `Edge` records that
compile together to one artifact. The flow-graph IS the
program.

## Test Commands

The repo carries shell scripts for the named test surfaces. Run
them through Nix so the pinned Rust toolchain is used:

```sh
nix run .#test
nix run .#test-kernel-surface
nix run .#test-legacy-slot-store
nix run .#test-doc
```

## License

[License of Non-Authority](LICENSE.md).

## Reference

`reference/Vision.md` — aspirational description
of the mature sema format (universal typed-binary of meaning,
self-transforming via quorum-signed spec changes, criome as the
distributed web). Reference material for future direction; current
implementation is the pseudo-sema layer above.
