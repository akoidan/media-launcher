class MediaLauncher < Formula
  desc "Matches audio/subtitle files to episodes and launches them in mpv or VLC"
  homepage "https://github.com/akoidan/media-launcher"
  version "0.1.2"

  on_arm do
    url "https://github.com/akoidan/media-launcher/releases/download/v0.1.2/media-launcher-macos-arm64"
    sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  end

  on_intel do
    url "https://github.com/akoidan/media-launcher/releases/download/v0.1.2/media-launcher-macos-x86_64"
    sha256 "0000000000000000000000000000000000000000000000000000000000000000"
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
