class MediaLauncher < Formula
  desc "Matches audio/subtitle files to episodes and launches them in mpv or VLC"
  homepage "https://github.com/akoidan/media-launcher"
  version "0.2.1"

  on_arm do
    url "https://github.com/akoidan/media-launcher/releases/download/v0.2.1/media-launcher-macos-arm64"
    sha256 "ef19e678c824acb0358c216ccf1b7ac0ae23f0dd38f492e180b4fca52782a6e9"
  end

  on_intel do
    url "https://github.com/akoidan/media-launcher/releases/download/v0.2.1/media-launcher-macos-x86_64"
    sha256 "de71636b3ff3064d9ed5c0fce7eb79ea6469456218ee8b2378c1fb43087d3575"
  end

  def install
    binary = Dir["media-launcher*"].reject { |f| f.end_with?(".rb") }.first
    bin.install binary => "media-launcher"
    chmod "+x", bin/"media-launcher"
  end

  test do
    assert_match "Usage", shell_output("#{bin}/media-launcher --help")
  end
end
