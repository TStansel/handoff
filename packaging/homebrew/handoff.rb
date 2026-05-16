class Handoff < Formula
  desc "Hand off local coding-agent context between Codex and Claude Code"
  homepage "https://github.com/TStansel/handoff"
  version "0.1.3"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/TStansel/handoff/releases/download/v0.1.3/handoff-v0.1.3-aarch64-apple-darwin.tar.gz"
      sha256 "3e8f125e7e9d0ffc3d9bf1b75b4070bcc96c249eb42c40f34d963aed84567e99"
    else
      url "https://github.com/TStansel/handoff/releases/download/v0.1.3/handoff-v0.1.3-x86_64-apple-darwin.tar.gz"
      sha256 "97671d47177b40769a14a7a5977ad57086c0d72d1d51e42989cb7f26c90e098c"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/TStansel/handoff/releases/download/v0.1.3/handoff-v0.1.3-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "9877ccc1eaf13625cc38a7c0098d8aabee19c245f5c82ec71e6b5502491eee5b"
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
