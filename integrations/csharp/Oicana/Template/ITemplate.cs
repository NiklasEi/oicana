using System.Text.Json.Nodes;
using Oicana.Interop;
using Oicana.Inputs;
using CompilationOptions = Oicana.Config.CompilationOptions;
using ExportOptions = Oicana.Config.ExportOptions;

namespace Oicana.Template;

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
    /// <param name="exportOptions">Options for the document export.</param>
    /// <param name="compilationOptions">Options for the template compilation.</param>
    /// <exception cref="OicanaException">If the template compilation fails.</exception>
    Stream Compile(IDictionary<string, JsonNode> jsonInputs, IDictionary<string, BlobInput> blobInputs, ExportOptions exportOptions, CompilationOptions compilationOptions);

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
