LC_ALL=C exec cloc \
  --exclude-dir=.tagit \
  --exclude-dir=.sqlx \
  --not-match-f=tuple.rs \
  --vcs=git
