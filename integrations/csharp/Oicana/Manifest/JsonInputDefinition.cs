using System.Text.Json.Serialization;

namespace Oicana.Manifest;

/// <summary>
/// An input taking a JSON value.
/// </summary>
public sealed class JsonInputDefinition : InputDefinition
{
    /// <summary>
    /// File in the template holding the value used when none is supplied.
    /// </summary>
    [JsonPropertyName("default")]
    public string? Default { get; init; }

    /// <summary>
    /// File in the template holding the value used in development mode when none is supplied.
    /// </summary>
    [JsonPropertyName("development")]
    public string? Development { get; init; }

    /// <summary>
    /// File in the template holding the JSON schema of this input.
    /// </summary>
    [JsonPropertyName("schema")]
    public string? Schema { get; init; }

    /// <summary>
    /// Whether values are validated against the schema.
    /// </summary>
    [JsonPropertyName("validate")]
    public required bool Validate { get; init; }
}
