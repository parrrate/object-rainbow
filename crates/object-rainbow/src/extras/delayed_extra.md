# Delayed extra inline injection refutation

I initially tried introducing delayed extra parsing as a structure which had `FetchBytes` and an
extra before it. This, however doesn't really work. Mostly the issue is that we need to `ListHashes`
during parsing already. And, since we don't have enough information to do that yet, it doesn't work.

With non-referring data this might work, but presently we don't provide `Extra` for those.
