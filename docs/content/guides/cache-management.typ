Typst uses #link("https://github.com/typst/comemo")[comemo], a memoized function cache. This significantly speeds up repeated compilations.

== How Cache Eviction Works

The comemo cache is global and shared across all template instances. To prevent unbounded memory growth, Oicana provides configurable cache eviction based on an aging mechanism:\
\
- Each cache entry has an age counter
- Age increases by 1 during each eviction call
- Age resets to 0 when the entry is accessed (cache hit)
- Entries with age ≥ max_age are removed when running cache eviction

== Default Behavior

By default, Oicana integrations automatically evict the cache after each compilation with a maximum age of 10. This means:\
\
- Cache entries used in the last 10 compilations are kept
- Older, unused entries are removed to free memory
- Most applications get good performance without manual tuning

== Configuring Cache Eviction

You can adjust the cache eviction age to match your application's needs. All integrations have APIs to
configure the maximum cache item age for eviction, completely turn of cache eviction, or trigger an eviction manually. 

=== When to Adjust automatic Cache Eviction

Consider adjusting the default cache settings to a higher maximum item age if you have a large number of templates and enough
available memory to support a larger cache.\
\
Sometimes, it can make sense to disable automatic cache eviction completely and run it manually. For example,
if you do large batch compilations of templates, it might improve performance, if you disable cache eviction during
batches of compilations and only clean up inbetween batches. This depends on your exact scenario. The default should be
usable in all scenarios, but you can test out different cache eviction settings for fine tuning.
