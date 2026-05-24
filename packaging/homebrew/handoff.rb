class Handoff < Formula
  desc "Hand off local coding-agent context between AI agent CLIs"
  homepage "https://github.com/TStansel/handoff"
  version "0.1.7"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/TStansel/handoff/releases/download/v0.1.7/handoff-v0.1.7-aarch64-apple-darwin.tar.gz"
      sha256 "81099c7d6cff8b7c836fa1cbb732237898f426f1fbf82ac09d24f459e4387f1e"
    else
      url "https://github.com/TStansel/handoff/releases/download/v0.1.7/handoff-v0.1.7-x86_64-apple-darwin.tar.gz"
      sha256 "743caea1536a206abe4e4dd59c83059a3b49e62ed19c4f3982b66dfb0cf6a3a7"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/TStansel/handoff/releases/download/v0.1.7/handoff-v0.1.7-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "fba591eaa5f02cbfcde8b12d8782501aa533a0fd430c9e283caa8299111651b8"
    else
      odie "Handoff does not currently publish a Linux ARM binary. Install from source with `cargo install --git https://github.com/TStansel/handoff`."
    end
  end

  def install
    bin.install "handoff"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/handoff --version")
    assert_match "Detected agents:", shell_output("#{bin}/handoff status")
    assert_match "# Handoff Packet", shell_output("#{bin}/handoff codex --dry-run")
  end
end
