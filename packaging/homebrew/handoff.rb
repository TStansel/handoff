class Handoff < Formula
  desc "Hand off local coding-agent context between Codex and Claude Code"
  homepage "https://github.com/TStansel/handoff"
  url "https://github.com/TStansel/handoff/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "9c5f21f446f68bd5f74bd74d8e3686de59d01009fe757f40b1858740288cca6d"
  license "MIT"
  head "https://github.com/TStansel/handoff.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "Handoff", shell_output("#{bin}/handoff --help")
  end
end
