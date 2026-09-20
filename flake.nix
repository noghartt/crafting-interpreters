{
  description = "Learning Lox with a Rust tree-walk interpreter and a Zig bytecode VM";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { nixpkgs, ... }:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          rustPackages = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer
          ];
          zigPackages = with pkgs; [
            zig
            zls
          ];
          rustSource = pkgs.rustPlatform.rustLibSrc;
        in
        {
          default = pkgs.mkShell {
            packages = rustPackages ++ zigPackages ++ [ pkgs.nixfmt ];
            RUST_SRC_PATH = rustSource;
          };

          rust = pkgs.mkShell {
            packages = rustPackages ++ [ pkgs.nixfmt ];
            RUST_SRC_PATH = rustSource;
          };

          zig = pkgs.mkShell {
            packages = zigPackages ++ [ pkgs.nixfmt ];
          };
        }
      );

      formatter = forAllSystems (system: nixpkgs.legacyPackages.${system}.nixfmt);
    };
}
