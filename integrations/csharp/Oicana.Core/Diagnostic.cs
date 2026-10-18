using System.Text.Json;
using System.Text.Json.Serialization;

namespace Oicana;

/// <summary>
/// A compilation diagnostic, like a warning, with its location in the template.
/// </summary>
/// <param name="Severity">Whether this is an error or a warning.</param>
/// <param name="Message">The message describing the problem.</param>
/// <param name="Location">Where in the template the problem is, or <c>null</c> if it does not point into a file.</param>
/// <param name="Hints">Hints on how to address the problem.</param>
/// <param name="Trace">The function calls leading to the problem, innermost first.</param>
/// <param name="Formatted">The diagnostic formatted for humans, with source excerpts and the configured diagnostic color.</param>
public sealed record Diagnostic(
    DiagnosticSeverity Severity,
    string Message,
    SourceLocation? Location,
    IReadOnlyList<string> Hints,
    IReadOnlyList<TracePoint> Trace,
    string Formatted)
{
    private static readonly JsonSerializerOptions Options = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        Converters = { new JsonStringEnumConverter(JsonNamingPolicy.CamelCase) },
    };

    /// <summary>
    /// The diagnostic formatted for humans.
    /// </summary>
    /// <returns><see cref="Formatted"/></returns>
    public override string ToString() => Formatted;

    /// <summary>
    /// Parse diagnostics from the JSON array the native library returns.
    /// </summary>
    /// <param name="json">The serialized diagnostics.</param>
    /// <exception cref="OicanaException">If the diagnostics cannot be parsed.</exception>
    /// <returns>The parsed diagnostics.</returns>
    internal static IReadOnlyList<Diagnostic> ListFromJson(string json)
    {
        try
        {
            return JsonSerializer.Deserialize<List<Diagnostic>>(json, Options)
                   ?? throw new OicanaException("The compilation warnings are empty");
        }
        catch (JsonException exception)
        {
            throw new OicanaException($"Failed to parse the compilation warnings: {exception.Message}");
        }
    }
}
