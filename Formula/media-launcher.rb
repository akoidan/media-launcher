class MediaLauncher < Formula
  desc "Matches audio/subtitle files to episodes and launches them in mpv or VLC"
  homepage "https://github.com/akoidan/media-launcher"
  url "https://github.com/akoidan/media-launcher.git", tag: "v0.1.2"
  version "0.1.2"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "Usage", shell_output("#{bin}/media-launcher --help")
  end
end
