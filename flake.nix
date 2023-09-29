{
  inputs = {
    geng.url = "github:geng-engine/cargo-geng";
  };
  outputs = { self, geng }: geng.makeFlakeOutputs (system:
    {
      src = geng.lib.${system}.filter {
        root = ./.;
        include = [
          "src"
          "Cargo.lock"
          "Cargo.toml"
        ];
      };
    });
}
