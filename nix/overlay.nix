final: prev: {
  comrak = final.callPackage ./package.nix { };
  comrak-www = final.callPackage ./www.nix { };
}
