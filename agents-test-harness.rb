class AgentsTestHarness < Formula
  desc "Test harness for validating AI agent behavior against steering guides"
  homepage "https://github.com/tatimblin/agent-execution-harness"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/tatimblin/agent-execution-harness/releases/download/v#{version}/harness-macos-arm64"
      sha256 "PLACEHOLDER_ARM64_SHA"
    else
      url "https://github.com/tatimblin/agent-execution-harness/releases/download/v#{version}/harness-macos-x86_64"
      sha256 "PLACEHOLDER_X86_64_SHA"
    end
  end

  on_linux do
    url "https://github.com/tatimblin/agent-execution-harness/releases/download/v#{version}/harness-linux-x86_64"
    sha256 "PLACEHOLDER_LINUX_SHA"
  end

  def install
    if OS.mac?
      if Hardware::CPU.arm?
        bin.install "harness-macos-arm64" => "harness"
      else
        bin.install "harness-macos-x86_64" => "harness"
      end
    else
      bin.install "harness-linux-x86_64" => "harness"
    end
  end

  test do
    system "#{bin}/harness", "--help"
  end
end