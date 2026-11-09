{
  description = "sema — Sema version 0: Markdown read structurally";

  inputs.base.url = "github:LiGoldragon/build-base";

  # Cargo.nix is generated from Cargo.lock by the build base's crate2nix and
  # committed, so evaluation imports no derivation:
  #   nix run github:LiGoldragon/build-base#crate2nix -- generate
  outputs =
    { self, base }:
    base.lib.workspaceOutputs {
      cargoNix = ./Cargo.nix;
      extraChecks =
        {
          pkgs,
          toolchain,
          buildRustCrateForPkgs,
          workspace,
          tests,
          ...
        }:
        let
          # The hand-written source; src/generated/ is ethos-zero's, proved
          # fresh by the build script.
          handWritten = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter =
              path: _:
              let
                relative = pkgs.lib.removePrefix (toString ./. + "/") path;
              in
              !(pkgs.lib.hasPrefix "src/generated" relative)
              && !(pkgs.lib.hasPrefix "measure" relative);
          };
          # A crate compiled by clippy-driver, every warning an error.
          clippyAttributes = {
            useClippy = true;
            clippy = toolchain;
            capLints = "forbid";
            extraRustcOpts = [
              "-D"
              "warnings"
            ];
          };
          # A member as the crate builder builds it, its dependencies the
          # same derivations as the build's, compiled by clippy-driver.
          linted =
            packageId:
            (workspace.internal.builtRustCratesWithFeatures {
              inherit packageId;
              features = [ "default" ];
              buildRustCrateForPkgsFunc = buildRustCrateForPkgs;
              runTests = false;
            }).crates.${packageId}.override
              clippyAttributes;
          grepRefuses =
            name: message: pattern: paths:
            pkgs.runCommand "sema-${name}" { } ''
              if grep -R -n -E '${pattern}' ${pkgs.lib.concatMapStringsSep " " (path: "${handWritten}/${path}") paths}; then
                echo "${message}" >&2
                exit 1
              fi
              touch $out
            '';
        in
        {
          # Each hand-written file read alone, so no module is followed.
          fmt = pkgs.runCommand "sema-fmt" { nativeBuildInputs = [ toolchain ]; } ''
            status=0
            for file in $(find ${handWritten} -name '*.rs'); do
              rustfmt --edition 2024 < "$file" | diff -u "$file" - || status=1
            done
            [ $status = 0 ] && touch $out
          '';
          clippy-sema = linted "sema";
          clippy-sema-tests = tests.sema.override clippyAttributes;
          clippy-sema-cli = linted "sema-cli";
          # No type or trait is declared by hand: every one is generated from the ethos.
          no-hand-written-declarations =
            grepRefuses "no-hand-written-declarations" "every type and trait is declared in the ethos"
              "^[[:space:]]*(pub(\\([^)]*\\))? )?(struct|enum|trait|type|union) "
              [
                "src"
                "cli"
              ];
          no-production-free-functions =
            grepRefuses "no-production-free-functions" "production Rust must not use module-level free functions"
              "^(pub(\\([^)]*\\))? )?fn "
              [ "src" ];
          no-production-inherent-methods =
            grepRefuses "no-production-inherent-methods" "production Rust must home behavior in traits"
              "^[[:space:]]*impl[[:space:]]+[[:alpha:]_][[:alnum:]_:<>]*[[:space:]]*\\{"
              [ "src" ];
        };
    };
}
