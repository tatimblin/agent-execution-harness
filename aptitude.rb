class Aptitude < Formula
  desc "Test harness for validating AI agent behavior against steering guides"
  homepage "https://github.com/tatimblin/aptitude"
  version "0.2.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/tatimblin/aptitude/releases/download/v#{version}/aptitude-macos-arm64"
      sha256 "PLACEHOLDER_ARM64_SHA"
    else
      url "https://github.com/tatimblin/aptitude/releases/download/v#{version}/aptitude-macos-x86_64"
      sha256 "PLACEHOLDER_X86_64_SHA"
    end
  end

  on_linux do
    url "https://github.com/tatimblin/aptitude/releases/download/v#{version}/aptitude-linux-x86_64"
    sha256 "PLACEHOLDER_LINUX_SHA"
  end

  def install
    if OS.mac?
      if Hardware::CPU.arm?
        bin.install "aptitude-macos-arm64" => "aptitude"
      else
        bin.install "aptitude-macos-x86_64" => "aptitude"
      end
    else
      bin.install "aptitude-linux-x86_64" => "aptitude"
    end
  end

  test do
    system "#{bin}/aptitude", "--help"
  end
end
