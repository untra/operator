#!/bin/sh
# Publish docs to gh-pages branch
# This script builds the Jekyll site and pushes to gh-pages using git subtree

set -e

# Clean up old build
rm -rf docs/_site/

# Build the Jekyll site
cd docs && bundle exec jekyll build && cd ..

# Add and commit the built site (may fail if nothing changed, that's ok)
git add docs/_site/ && git commit -m "docs: build site $(date '+%Y-%m-%d %H:%M:%S')" || true

# Push the docs/_site directory to gh-pages branch
git subtree push --prefix docs/_site origin gh-pages
