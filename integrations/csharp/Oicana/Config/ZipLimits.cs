namespace Oicana.Config;

/// <summary>
/// Limits applied when reading a packed template zip.
///
/// Leaving a bound as <c>null</c> keeps the default
/// (10 000 entries / 512 MiB decompressed).
/// </summary>
public class ZipLimits
{
    /// <summary>Maximum number of zip entries, or <c>null</c> for the default.</summary>
    public long? MaxEntries { get; init; }

    /// <summary>Maximum total decompressed size in bytes, or <c>null</c> for the default.</summary>
    public long? MaxTotalDecompressedBytes { get; init; }
}
