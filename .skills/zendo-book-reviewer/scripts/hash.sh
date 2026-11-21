#!/bin/sh
# Print "sha256:<hash>" of a book file with an empty value in its `hash` field.
# The hash covers the whole file, the review data included. tests/book_review.rs computes the same
# hash.
set -eu
file=$1
text=$(sed 's/^  hash:.*$/  hash: ""/' "$file"; printf x)
if command -v sha256sum >/dev/null 2>&1; then
    sum="sha256sum"
else
    sum="shasum -a 256"
fi
hash=$(printf '%s' "${text%x}" | $sum | cut -d' ' -f1)
echo "sha256:$hash"
