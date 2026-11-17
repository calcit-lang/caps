# Verify the actual outdated cache paths

The cache root is a sibling of the configured module directory and startup writes
its AGENTS.md even when the prompt is declined. Check for downloaded git
revisions and the project-local module view instead of treating that documented
startup metadata write as a download.
