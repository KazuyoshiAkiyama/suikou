# Invalidating a cache

A cache stores the result of an expensive computation so that the same input does not
have to be computed again. The heavier the computation, the more a cache helps. The cost
is that the stored result can go stale when the underlying data changes.

There are two broad approaches to invalidation. One is to expire entries after a fixed
time. The other is to detect changes and drop the affected entries.

Expiring by time is simple to implement, but choosing the interval is hard. A short
interval reduces the benefit of caching. A long interval widens the window in which a
stale result can be served.

Detecting changes is accurate, but you have to build a path for the notification to
travel. Some systems use database triggers. Others call the invalidation explicitly from
the code that performs the update. If even one path is missed, stale results leak through
it, much like a slow leak in a pipe.

In practice the two are combined. Change detection does the work, and a generous
expiry time acts as a backstop.

## Choosing an expiry interval

The interval controls a trade-off that has no general answer. It depends on how often the
underlying data changes and on how much a stale answer costs. For a product catalogue that
is updated once a day, an interval of an hour is generous. For a price that moves every
second, an hour is unusable.

A useful way to choose is to measure. Record how often the underlying data changes, and
look at the distribution rather than the average. If most changes arrive in bursts, an
interval tuned to the average will be wrong most of the time.

## Keys and their shape

The key determines what counts as the same input. A key that is too coarse returns a
result computed for a different input. A key that is too fine never gets a hit, and the
cache costs more than it saves.

Include every input that affects the result, and nothing else. Request identifiers, trace
identifiers, and timestamps usually belong in the second category. A common failure is to
build the key from the whole request object, which silently disables the cache.

## Measuring whether it helps

A cache that is never hit is worse than no cache at all, because it adds a lookup and a
store to every request. Record the hit rate from the start. Record the latency of the
lookup as well, because a remote cache can be slower than recomputing a cheap result.

Compare the total time with and without the cache under a realistic load. A benchmark that
replays the same request repeatedly will report a hit rate close to one, which tells you
nothing about production.
