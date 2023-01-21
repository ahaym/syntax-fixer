{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell {
    nativeBuildInputs = [ pkgs.openssl pkgs.cargo pkgs.nodejs ];
}
