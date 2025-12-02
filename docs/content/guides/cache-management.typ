Typst uses #link("https://github.com/typst/comemo")[comemo], a memoized function cache. This significantly speeds up repeated compilations.

== How Cache Eviction Works

The comemo cache is global and shared across all template instances. To prevent unbounded memory growth, Oicana provides configurable cache eviction based on an aging mechanism:\
\
- Each cache entry has an age counter
- Age increases by 1 during each eviction call
- Age resets to 0 when the entry is accessed (during a template compilation)
- Entries with age ≥ max_age are removed when running cache eviction

== Default Behavior

By default, Oicana integrations automatically evict the cache after each compilation with a maximum age of 10.

== Configuring Cache Eviction

All integrations provide two APIs for cache management:

- `configureAutomaticCacheEviction(age)` - Configure or disable automatic eviction after each compilation
- `evictCache(age)` - Manually trigger cache eviction with a specific age threshold
\

Consider adjusting the default cache settings to a higher maximum entry age if you have a large number of templates and enough
available memory to support a larger cache.\
\
Sometimes, it makes sense to disable automatic cache eviction and only run it manually. For example,
during large batch compilations, you can disable automatic eviction and only evict between batches for better performance.
The default should work well in most scenarios, but you can experiment with different approaches for fine-tuning.
