using System.Text.Json.Nodes;
using Oicana.Interop;
using Oicana.Inputs;
using CompilationOptions = Oicana.Config.CompilationOptions;
using ExportFormat = Oicana.Config.ExportFormat;
using PageRange = Oicana.Config.PageRange;

namespace Oicana;

/// <summary>
/// An Oicana template
/// </summary>
public interface ITemplate
{
    /// <summary>
    /// Compile the template with the given inputs to the specified format.
    /// </summary>
    /// <param name="jsonInputs">Json inputs for the compilation (key -> JsonNode).</param>
    /// <param name="blobInputs">Blob inputs for the compilation (key -> BlobInput).</param>
    /// <param name="exportFormat">Format configuration for the document export.</param>
    /// <param name="compilationOptions">Options for the template compilation.</param>
    /// <param name="pages">0-based, inclusive page range to export, or <c>null</c> for the whole document.</param>
    /// <exception cref="OicanaException">If the template compilation fails.</exception>
    Stream Export(IDictionary<string, JsonNode> jsonInputs, IDictionary<string, BlobInput> blobInputs, ExportFormat exportFormat, CompilationOptions compilationOptions, PageRange? pages = null);

    /// <summary>
    /// Compile the template with the given inputs and export it to PDF.
    /// </summary>
    /// <param name="jsonInputs">Json inputs for the compilation (key -> JsonNode).</param>
    /// <param name="blobInputs">Blob inputs for the compilation (key -> BlobInput).</param>
    /// <param name="compilationOptions">Options for the template compilation.</param>
    /// <param name="pages">0-based, inclusive page range to export, or <c>null</c> for the whole document.</param>
    /// <exception cref="OicanaException">If the template compilation fails.</exception>
    Stream ExportPdf(IDictionary<string, JsonNode> jsonInputs, IDictionary<string, BlobInput> blobInputs, CompilationOptions compilationOptions, PageRange? pages = null);

    /// <summary>
    /// Compile the template with the given inputs and export it to PNG.
    /// Multiple pages are merged into a single, vertically stacked image.
    /// </summary>
    /// <param name="jsonInputs">Json inputs for the compilation (key -> JsonNode).</param>
    /// <param name="blobInputs">Blob inputs for the compilation (key -> BlobInput).</param>
    /// <param name="compilationOptions">Options for the template compilation.</param>
    /// <param name="pixelsPerPt">Resolution in pixels per point (defaults to 1.0).</param>
    /// <param name="pages">0-based, inclusive page range to export, or <c>null</c> for the whole document.</param>
    /// <exception cref="OicanaException">If the template compilation fails.</exception>
    Stream ExportPng(IDictionary<string, JsonNode> jsonInputs, IDictionary<string, BlobInput> blobInputs, CompilationOptions compilationOptions, float pixelsPerPt = 1.0f, PageRange? pages = null);

    /// <summary>
    /// Compile the template with the given inputs and export it to SVG.
    /// </summary>
    /// <param name="jsonInputs">Json inputs for the compilation (key -> JsonNode).</param>
    /// <param name="blobInputs">Blob inputs for the compilation (key -> BlobInput).</param>
    /// <param name="compilationOptions">Options for the template compilation.</param>
    /// <param name="pages">0-based, inclusive page range to export, or <c>null</c> for the whole document.</param>
    /// <exception cref="OicanaException">If the template compilation fails.</exception>
    Stream ExportSvg(IDictionary<string, JsonNode> jsonInputs, IDictionary<string, BlobInput> blobInputs, CompilationOptions compilationOptions, PageRange? pages = null);

    /// <summary>
    /// Compile the template and return the compiled document.
    ///
    /// Unlike <see cref="Export"/>, the document is kept in memory so it can be exported one or
    /// more times without re-compiling. Dispose
    /// the returned document (or use a <c>using</c> statement) to free it.
    /// </summary>
    /// <param name="jsonInputs">Json inputs for the compilation (key -> JsonNode).</param>
    /// <param name="blobInputs">Blob inputs for the compilation (key -> BlobInput).</param>
    /// <param name="compilationOptions">Options for the template compilation.</param>
    /// <exception cref="OicanaException">If the template compilation fails.</exception>
    /// <returns>A handle to the compiled document.</returns>
    CompiledDocument Compile(IDictionary<string, JsonNode> jsonInputs, IDictionary<string, BlobInput> blobInputs, CompilationOptions compilationOptions);

    /// <summary>
    /// Enable or disable JSON schema validation for this template.
    ///
    /// When enabled (the default), JSON inputs are validated against their schemas
    /// before compilation.
    /// </summary>
    /// <param name="validate">Whether to validate inputs against their JSON schemas.</param>
    void SetValidateInputs(bool validate);

    /// <summary>
    /// Get input definitions from the template manifest.
    /// </summary>
    /// <exception cref="OicanaException">If inputs cannot be retrieved.</exception>
    /// <returns>JSON string containing input definitions.</returns>
    string Inputs();

    /// <summary>
    /// Get source file content from the template.
    /// </summary>
    /// <param name="path">File path in the template.</param>
    /// <exception cref="OicanaException">If the file cannot be found.</exception>
    /// <returns>Source file content as a string.</returns>
    string Source(string path);

    /// <summary>
    /// Get binary file content from the template.
    /// </summary>
    /// <param name="path">File path in the template.</param>
    /// <exception cref="OicanaException">If the file cannot be found.</exception>
    /// <returns>Binary file content as a byte array.</returns>
    byte[] File(string path);
}
