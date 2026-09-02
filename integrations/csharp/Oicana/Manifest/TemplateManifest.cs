using System.Text.Json;
using System.Text.Json.Serialization;

namespace Oicana.Manifest;

/// <summary>
/// A template's manifest.
/// </summary>
public sealed class TemplateManifest
{
    /// <summary>
    /// The Typst package section of the manifest.
    /// </summary>
    [JsonPropertyName("package")]
    public required PackageInfo Package { get; init; }

    /// <summary>
    /// The Oicana section of the manifest.
    /// </summary>
    [JsonPropertyName("oicana")]
    public required OicanaConfig Oicana { get; init; }

    /// <summary>
    /// Parse a manifest from the JSON the native library returns.
    /// </summary>
    /// <param name="json">The serialized manifest.</param>
    /// <exception cref="OicanaException">If the manifest cannot be parsed.</exception>
    /// <returns>The parsed manifest.</returns>
    public static TemplateManifest FromJson(string json)
    {
        try
        {
            return JsonSerializer.Deserialize<TemplateManifest>(json)
                   ?? throw new OicanaException("The template manifest is empty");
        }
        catch (JsonException exception)
        {
            throw new OicanaException($"Failed to parse the template manifest: {exception.Message}");
        }
    }
}
