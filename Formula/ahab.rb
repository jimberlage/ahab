class Ahab < Formula
  desc "ahab CLI tool"
  homepage "https://github.com/jimberlage/ahab"
  version "2.0.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/jimberlage/ahab/releases/download/v1.0.0/ahab-aarch64-apple-darwin.tar.gz"
      sha256 "ae9abaf5ade888ea5c3e98534fef9681108ce40db4a902e803ea7115767e18d2"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/jimberlage/ahab/releases/download/v1.0.0/ahab-aarch64-linux.tar.gz"
      sha256 "88a01625387c3d8610f437ec061b74f24b7e06f9669874b029106021ee37965d"
    end
    on_intel do
      url "https://github.com/jimberlage/ahab/releases/download/v1.0.0/ahab-x86_64-linux.tar.gz"
      sha256 "51321af8916abe67b07db12ccc98c85e1caac664156b339769fd7677c7150b0e"
    end
  end

  def install
    bin.install "ahab"
  end

  test do
    system "#{bin}/ahab", "--version"
  end
end