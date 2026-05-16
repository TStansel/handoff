class Handoff < Formula
  desc "Hand off local coding-agent context between Codex and Claude Code"
  homepage "https://github.com/TStansel/handoff"
  version "0.1.2"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/TStansel/handoff/releases/download/v0.1.2/handoff-v0.1.2-aarch64-apple-darwin.tar.gz"
      sha256 "63481698b317770581d6b6403665d53b8c945203c9f59ff84fd00241175ac7f3"
    else
      url "https://github.com/TStansel/handoff/releases/download/v0.1.2/handoff-v0.1.2-x86_64-apple-darwin.tar.gz"
      sha256 "31943c202d2b64c87492969d3688011397188ef33dd8e5d0198ac21224e202bb"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/TStansel/handoff/releases/download/v0.1.2/handoff-v0.1.2-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "14a1ac7266eecc16584e226f12f7efcb2eafddc91dd7a2b6c674c44b38742513"
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
