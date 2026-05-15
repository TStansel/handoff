class Handoff < Formula
  desc "Hand off local coding-agent context between Codex and Claude Code"
  homepage "https://github.com/TStansel/handoff"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/TStansel/handoff/releases/download/v0.1.0/handoff-v0.1.0-aarch64-apple-darwin.tar.gz"
      sha256 "43589d52ca7c60b680569d4fd219b34d7598893e61726cdd1cb1359e3c67fcfbb56"
    else
      odie "Handoff does not currently publish an Intel macOS binary. Install from source with `cargo install --git https://github.com/TStansel/handoff`."
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/TStansel/handoff/releases/download/v0.1.0/handoff-v0.1.0-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "7ed1e70127800b71f3df48cd9f8b64d6967f2027169ea5a6bae126cbe1d54c60"
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
