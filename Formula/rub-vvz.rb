class RubVvz < Formula
  desc "CLI for the public Ruhr-Universitaet Bochum course catalogue"
  homepage "https://github.com/TobiasPol/rub-vvz-cli"
  url "https://github.com/TobiasPol/rub-vvz-cli/archive/refs/tags/v0.1.0-beta.1.tar.gz"
  sha256 "ea99cb9365af74a97015666b1a7d8cd253227bf1e97a0cb78f754c41d10a83ea"
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
