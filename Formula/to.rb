class To < Formula
  desc "Exploratory zsh directory jumper"
  homepage "https://github.com/Z-ready/zsh-to"
  url "https://github.com/Z-ready/zsh-to/archive/refs/tags/v1.1.4.tar.gz"
  sha256 "adb835bc7e7810941a8bc2af5bda3cfbb194b0c3f194700e585ab5ecfd8e27d3"
  version "1.1.4"
  license "MIT"

  depends_on "rust" => :build
  depends_on "fd"
  depends_on "fzf"
  depends_on "sqlite"

  on_macos do
    depends_on "fswatch"
  end

  on_linux do
    depends_on "inotify-tools"
  end

  def install
    system "cargo", "install", *std_cargo_args
    bin.install "bin/to"
    pkgshare.install "to.plugin.zsh"
    zsh_completion.install "completions/_to"
    doc.install "README.md"
    prefix.install "LICENSE"
  end

  def caveats
    <<~EOS
      Add this to your ~/.zshrc:

        eval "$(to init zsh)"

      Then reload zsh and configure search roots:

        source ~/.zshrc
        to use ~/Projects
        to roots
    EOS
  end

  test do
    assert_path_exists bin/"to"
    assert_path_exists bin/"to-helper"
    assert_match "source #{pkgshare}/to.plugin.zsh", shell_output("#{bin}/to init zsh")
    system "zsh", "-n", "#{pkgshare}/to.plugin.zsh"
    assert_match "to config:", shell_output("zsh -fc 'source #{pkgshare}/to.plugin.zsh; to --doctor'")
  end
end
