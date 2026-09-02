using System.Text.Json.Nodes;
using System.Text.Json.Serialization;

namespace Oicana.Manifest;

/// <summary>
/// A blob from the template, used when no value is supplied.
/// </summary>
public sealed class BlobFallback
{
    /// <summary>
    /// File in the template holding the blob.
    /// </summary>
    [JsonPropertyName("file")]
    public required string File { get; init; }

    /// <summary>
    /// Metadata passed to the template along with the blob.
    /// </summary>
    [JsonPropertyName("meta")]
    public JsonObject? Meta { get; init; }
}
