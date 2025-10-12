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
    /// <param name="jsonInputs">Json inputs for the compilation.</param>
    /// <param name="blobInputs">Blob inputs for the compilation.</param>
    /// <param name="compilationOptions">Options for the template compilation.</param>
    /// <param name="exportOptions">Options for the document export.</param>
    /// <exception cref="OicanaException">If the template compilation fails.</exception>
    Stream Compile(IList<TemplateJsonInput> jsonInputs, IList<TemplateBlobInput> blobInputs, CompilationOptions compilationOptions, ExportOptions exportOptions);
}
