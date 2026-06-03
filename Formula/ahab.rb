class Ahab < Formula
  desc "ahab CLI tool"
  homepage "https://github.com/jimberlage/ahab"
  version "4.0.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/jimberlage/ahab/releases/download/v4.0.0/ahab-aarch64-apple-darwin.tar.gz"
      sha256 "6607440df8a81cf99197ee788dc8ae482f908b373080aa6fb9f6b2b586557d8d"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/jimberlage/ahab/releases/download/v4.0.0/ahab-aarch64-linux.tar.gz"
      sha256 "503d206ffdbc2489b76deea62fac102f0c19102bb67b690cc5ac6d0295370309"
    end
    on_intel do
      url "https://github.com/jimberlage/ahab/releases/download/v4.0.0/ahab-x86_64-linux.tar.gz"
      sha256 "326e64cd606bd34368c8f4caa31389139e07b3bff6f7f752ccb06920b896c0f1"
    end
  end

  def install
    bin.install "ahab"
  end

  test do
    system "#{bin}/ahab", "--version"
  end
end