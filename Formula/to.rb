class To < Formula
  desc "Exploratory zsh directory jumper"
  homepage "https://github.com/yourname/zsh-to"
  url "file:///Users/z-ready/i/zsh-to/dist/to-0.1.0.tar.gz"
  sha256 "d44e1397bda853a79faba40997b61927f213029321521076f1e0565b90652d3e"
  version "0.1.0"
  license "MIT"

  depends_on "fd" => :recommended
  depends_on "fzf" => :optional

  def install
    pkgshare.install "to.plugin.zsh"
    zsh_completion.install "completions/_to"
    doc.install "README.md"
    prefix.install "LICENSE"
  end

  def caveats
    <<~EOS
      Add this to your ~/.zshrc:

        source "#{opt_pkgshare}/to.plugin.zsh"

      Then reload zsh and configure search roots:

        source ~/.zshrc
        to use ~/Projects
        to roots
    EOS
  end

  test do
    system "zsh", "-n", "#{pkgshare}/to.plugin.zsh"
    assert_match "to config:", shell_output("zsh -fc 'source #{pkgshare}/to.plugin.zsh; to --doctor'")
  end
end
