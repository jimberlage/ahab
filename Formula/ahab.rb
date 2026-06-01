class Ahab < Formula
  desc "ahab CLI tool"
  homepage "https://github.com/jimberlage/ahab"
  version "1.0.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/jimberlage/ahab/releases/download/v1.0.0/ahab-aarch64-apple-darwin.tar.gz"
      sha256 "<CHANGE ME>"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/jimberlage/ahab/releases/download/v1.0.0/ahab-aarch64-linux.tar.gz"
      sha256 "<CHANGE ME>"
    end
    on_intel do
      url "https://github.com/jimberlage/ahab/releases/download/v1.0.0/ahab-x86_64-linux.tar.gz"
      sha256 "<CHANGE ME>"
    end
  end

  def install
    bin.install "ahab"
  end

  test do
    system "#{bin}/ahab", "--version"
  end
end