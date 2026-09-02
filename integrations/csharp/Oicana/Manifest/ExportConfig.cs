using System.Text.Json.Serialization;

namespace Oicana.Manifest;

/// <summary>
/// How compiled documents are exported.
/// </summary>
public sealed class ExportConfig
{
    /// <summary>
    /// PDF export configuration.
    /// </summary>
    [JsonPropertyName("pdf")]
    public required PdfExportConfig Pdf { get; init; }
}
