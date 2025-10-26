{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    python311
    git
    gnumake
    gh
    wasm-pack
  ];

  shellHook = ''
  '';
}
