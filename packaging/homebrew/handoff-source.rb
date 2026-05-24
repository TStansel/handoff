class HandoffSource < Formula
  desc "Hand off local coding-agent context between AI agent CLIs"
  homepage "https://github.com/TStansel/handoff"
  url "https://github.com/TStansel/handoff/archive/refs/tags/v0.1.6.tar.gz"
  sha256 "9a9a679d09dd0f56339836a997786ffca9172f17a5d16a1305a10f4746d9ac36"
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
