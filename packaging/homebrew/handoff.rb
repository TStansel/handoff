class Handoff < Formula
  desc "Hand off local coding-agent context between AI agent CLIs"
  homepage "https://github.com/TStansel/handoff"
  version "0.1.5"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/TStansel/handoff/releases/download/v0.1.5/handoff-v0.1.5-aarch64-apple-darwin.tar.gz"
      sha256 "a865f21832dce702449fbbfb3cabc84fd33f685c4833febdd44a62b9f8e18b01"
    else
      url "https://github.com/TStansel/handoff/releases/download/v0.1.5/handoff-v0.1.5-x86_64-apple-darwin.tar.gz"
      sha256 "7c50eeb56b9f79220289c9c8b55cc226753e23c4476e3c81b622118c2fc8edc8"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/TStansel/handoff/releases/download/v0.1.5/handoff-v0.1.5-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "02319a11e917c6bdf8672a583904b8fb706bb1cf6c36554bfa3c3bd7ee61bb27"
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
