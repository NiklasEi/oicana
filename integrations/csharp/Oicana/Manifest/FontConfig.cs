using System.Text.Json.Serialization;

namespace Oicana.Manifest;

/// <summary>
/// Fonts a template expects from its host.
/// </summary>
public sealed class FontConfig
{
    /// <summary>
    /// Font families the host has to register.
    /// </summary>
    [JsonPropertyName("require")]
    public required IReadOnlyList<string> Require { get; init; }
}
