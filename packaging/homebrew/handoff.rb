class Handoff < Formula
  desc "Hand off local coding-agent context between AI agent CLIs"
  homepage "https://github.com/TStansel/handoff"
  version "0.1.6"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/TStansel/handoff/releases/download/v0.1.6/handoff-v0.1.6-aarch64-apple-darwin.tar.gz"
      sha256 "e43853d9471e62cf2a90c27a91b932a3bd7587f915d326920a076272317f2fd6"
    else
      url "https://github.com/TStansel/handoff/releases/download/v0.1.6/handoff-v0.1.6-x86_64-apple-darwin.tar.gz"
      sha256 "d9d52077aad98b302c6cf8f444d691e1ce1c7e4d54938af6a862625d9cb4c0af"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/TStansel/handoff/releases/download/v0.1.6/handoff-v0.1.6-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "0c333742b5d36f8f675061294fe14867fb38963d7e98674354a893a29f0649ba"
    else
      odie "Handoff does not currently publish a Linux ARM binary. Install from source with `cargo install --git https://github.com/TStansel/handoff`."
    end
  end

  def install
    bin.install "handoff"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/handoff --version")
  end
end
