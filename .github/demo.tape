# SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# Renders .github/demo.gif with VHS (https://github.com/charmbracelet/vhs).
# Run `make demo`, which builds the examples first so no compile output
# is recorded. Everything shown runs locally against the fixture in
# .github/demo/.

Output .github/demo.gif
Set Shell "bash"
Set FontSize 16
Set Width 1100
Set Height 620
Set Padding 20
Set TypingSpeed 35ms

Hide
Type "export PS1='$ ' && clear"
Enter
Show

Type "cat .github/demo/post.md"
Enter
Sleep 1.5s

Type "./.github/demo/extract.sh .github/demo/post.md"
Enter
Sleep 2.5s

