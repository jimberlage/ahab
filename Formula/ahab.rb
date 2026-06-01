class Ahab < Formula
  desc "ahab CLI tool"
  homepage "https://github.com/jimberlage/ahab"
  version "1.0.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/jimberlage/ahab/releases/download/v1.0.0/ahab-aarch64-apple-darwin.tar.gz"
      sha256 "93368a3f7413d5f69968a2f58fe5a94b379fd262d2915f8beef6d8375a34fe48"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/jimberlage/ahab/releases/download/v1.0.0/ahab-aarch64-linux.tar.gz"
      sha256 "658020e2afa200306b8dab66da17f9acc3833d813d5bf9b84fba271f53cdb97e"
    end
    on_intel do
      url "https://github.com/jimberlage/ahab/releases/download/v1.0.0/ahab-x86_64-linux.tar.gz"
      sha256 "d2a0c488dce1bbc3d3ba17121f7e482ca0686c6011eac5ed42a108b6ca308f95"
    end
  end

  def install
    bin.install "ahab"
  end

  test do
    system "#{bin}/ahab", "--version"
  end
end