class Handoff < Formula
  desc "Hand off local coding-agent context between Codex and Claude Code"
  homepage "https://github.com/TStansel/handoff"
  version "0.1.1"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/TStansel/handoff/releases/download/v0.1.1/handoff-v0.1.1-aarch64-apple-darwin.tar.gz"
      sha256 "53ee7d76b042cf388ab8b2bca5ed56371f6aeb397c94f8eb83ec9cb8aaf4d4a0"
    else
      odie "Handoff does not currently publish an Intel macOS binary. Install from source with `cargo install --git https://github.com/TStansel/handoff`."
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/TStansel/handoff/releases/download/v0.1.1/handoff-v0.1.1-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "5f721b63c7cf2c351c645131be30e6b228c8ea907c4b942f8955a32bdcb8e075"
    else
      odie "Handoff does not currently publish a Linux ARM binary. Install from source with `cargo install --git https://github.com/TStansel/handoff`."
    end
  end

  def install
    bin.install "handoff"
  end

  test do
    assert_match "Handoff", shell_output("#{bin}/handoff --help")
  end
end
