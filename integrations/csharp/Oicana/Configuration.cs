using Oicana.Interop;

namespace Oicana;

/// <summary>
/// Global Oicana configuration.
/// </summary>
public static class Configuration
{
    /// <summary>
    /// Configure Oicana.
    /// </summary>
    /// <param name="coloring">Coloring for Oicana diagnostics.</param>
    public static void Configure(DiagnosticsColoring coloring)
    {
        OicanaFfi.Configure(coloring);
    }
}
