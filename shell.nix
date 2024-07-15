with import <nixpkgs> { };
stdenv.mkDerivation {
  name = "frameless";
  buildInputs = [ pkg-config gtk4 libsoup_3 webkitgtk_6_0 libadwaita ];
}
