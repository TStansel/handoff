class Handoff < Formula
  desc "Hand off local coding-agent context between Codex and Claude Code"
  homepage "https://github.com/TStansel/handoff"
  version "0.1.4"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/TStansel/handoff/releases/download/v0.1.4/handoff-v0.1.4-aarch64-apple-darwin.tar.gz"
      sha256 "022459a6986db84c8d63c2d06d20f2d4bf6eb69df4d3e51ddb7c4098b7717f0d"
    else
      url "https://github.com/TStansel/handoff/releases/download/v0.1.4/handoff-v0.1.4-x86_64-apple-darwin.tar.gz"
      sha256 "e7739e18e43794429ef338e7892648d27ffc6d8f9edb5a3d5a56d6c07f3789b9"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/TStansel/handoff/releases/download/v0.1.4/handoff-v0.1.4-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "34995d1b2b824956406fc76f0134fd1a23f9aab331b308fab19eb53f2607f274"
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
