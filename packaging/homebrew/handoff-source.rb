class HandoffSource < Formula
  desc "Hand off local coding-agent context between AI agent CLIs"
  homepage "https://github.com/TStansel/handoff"
  url "https://github.com/TStansel/handoff/archive/refs/tags/v0.1.7.tar.gz"
  sha256 "1ab34faeb1a7c13804606334981215d39fe6b57ccfc0394f4bdedb8fcc15c316"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/handoff --version")
    assert_match "Detected agents:", shell_output("#{bin}/handoff status")
    assert_match "# Handoff Packet", shell_output("#{bin}/handoff codex --dry-run")
  end
end
