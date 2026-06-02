class RubVvz < Formula
  desc "CLI for the public Ruhr-Universitaet Bochum course catalogue"
  homepage "https://github.com/TobiasPol/rub-vvz-cli"
  url "https://github.com/TobiasPol/rub-vvz-cli/archive/refs/tags/v0.1.0-beta.2.tar.gz"
  sha256 "c91aef1280b09e090f92d912f9b5e4e8b1bbfba8dc89d147633dc555e004d76e"
  license "MIT"

  depends_on "rust" => :build
  depends_on "curl"

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "rub-vvz", shell_output("#{bin}/rub-vvz --help")
  end
end
