class MediaLauncher < Formula
  desc "Matches audio/subtitle files to episodes and launches them in mpv or VLC"
  homepage "https://github.com/akoidan/media-launcher"
  version "0.2.2"

  on_arm do
    url "https://github.com/akoidan/media-launcher/releases/download/v0.2.2/media-launcher-macos-arm64"
    sha256 "e74817d3535c3d7e4882df22570a92cffd443ba47616ea3077fb6ed946cf93d8"
  end

  on_intel do
    url "https://github.com/akoidan/media-launcher/releases/download/v0.2.2/media-launcher-macos-x86_64"
    sha256 "d12f30a4b5ad2bff4b9feb9e0ba2941ca3defbeaafa80e0f5ce4b61581a07c4b"
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
