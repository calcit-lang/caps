# Outdated confirmation at EOF

`caps outdated` prints a default-no prompt before updating the manifest, but
previously treated an empty response as approval. Closed stdin and a blank line
therefore silently upgraded dependencies. Keep the existing command and `-y`
behavior, and accept only an explicit case-insensitive `y` or `yes` at the prompt.

The CLI regression runs against a temporary custom manifest with no dependencies
and an outdated toolchain version. It checks EOF, blank and negative responses
without downloads or writes, and checks both affirmative responses and `-y` while
preserving the project version. No network access or real user manifest is needed.
