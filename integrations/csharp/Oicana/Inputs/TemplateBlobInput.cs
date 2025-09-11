using System.Diagnostics.CodeAnalysis;
using System.Text.Json.Nodes;

namespace Oicana.Inputs;

/// <summary>
/// A blob input
/// </summary>
public class TemplateBlobInput
{
    /// <summary>
    /// Construct a new input from a key and bytes
    /// </summary>
    [SetsRequiredMembers]
    public TemplateBlobInput(string key, byte[] value) : this(key, value, null) { }

    /// <summary>
    /// Construct a new input from a key and bytes
    /// </summary>
    [SetsRequiredMembers]
    public TemplateBlobInput(string key, byte[] value, BlobMeta? meta)
    {
        Key = key;
        Blob = value;
        Meta = meta?.Build();
    }

    /// <summary>
    /// key for the blob input
    /// </summary>
    /// <example>logo</example>
    public required string Key { get; init; }

    /// <summary>
    /// binary blob
    /// </summary>
    public required byte[] Blob { get; init; }

    /// <summary>
    /// meta data for the blob
    /// </summary>
    public JsonNode? Meta { get; init; }
}
