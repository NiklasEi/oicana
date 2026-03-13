using Oicana.Interop;

namespace Oicana;

/// <summary>
/// Global Oicana configuration.
/// </summary>
public static class Configuration
{
    /// <summary>
    /// Configure diagnostics coloring for Oicana.
    /// </summary>
    /// <param name="coloring">Coloring for Oicana diagnostics.</param>
    public static void DiagnosticsColoring(DiagnosticsColoring coloring)
    {
        OicanaFfi.Configure(coloring);
    }

    /// <summary>
    /// Configure automatic cache eviction after each compilation.
    ///
    /// Each cache entry has an age counter that increases by 1 during each eviction
    /// and resets to 0 on cache hit. Entries with age >= maxAge are removed.
    /// </summary>
    /// <param name="maxAge">
    /// Maximum age threshold, or -1 to disable:
    ///   - -1 - Disables cache eviction (cache never cleared)
    ///   - 0 - Clears all cache after every compilation
    ///   - 1 - Keeps only entries used since the last eviction
    ///   - n - Keeps entries used within the last n evictions
    /// Default is 10.
    /// </param>
    public static void ConfigureAutomaticCacheEviction(long maxAge)
    {
        OicanaFfi.ConfigureAutomaticCacheEviction(maxAge);
    }

}
