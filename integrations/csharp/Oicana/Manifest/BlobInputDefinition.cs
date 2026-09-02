using System.Text.Json.Serialization;

namespace Oicana.Manifest;

/// <summary>
/// An input taking arbitrary bytes.
/// </summary>
public sealed class BlobInputDefinition : InputDefinition
{
    /// <summary>
    /// Blob used when no value is supplied.
    /// </summary>
    [JsonPropertyName("default")]
    public BlobFallback? Default { get; init; }

    /// <summary>
    /// Blob used in development mode when no value is supplied.
    /// </summary>
    [JsonPropertyName("development")]
    public BlobFallback? Development { get; init; }
}
