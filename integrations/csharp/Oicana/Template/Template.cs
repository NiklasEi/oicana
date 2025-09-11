using Oicana.Interop;
using Oicana.Inputs;
using CompilationMode = Oicana.Config.CompilationMode;
using CompilationOptions = Oicana.Config.CompilationOptions;

namespace Oicana.Template;

/// <summary>
/// An Oicana template
/// 
/// This class caches the template file and prepares fast
/// compilation as part of the constructor. Keep it around
/// to compile the same template with different inputs.
/// </summary>
public class Template : ITemplate, IDisposable
{
    private readonly string _templateId;

    /// <summary>
    /// Prepare a template for fast compilation.
    ///
    /// This will compile the document in development mode.
    /// Your template should not have any required inputs that don't have
    /// development or default values defined, otherwise registration will fail.
    /// 
    /// This call can be expensive depending on the template.
    /// Reuse instances of this class if possible.
    /// 
    /// If you want to compile a template once and not cache the template
    /// use <see cref="CompileOnce"/> instead.
    /// </summary>
    /// <param name="templateFile">The packed Oicana template to register.</param>
    /// <exception cref="OicanaException">If the initial template compilation fails.</exception>
    public Template(byte[] templateFile) : this(templateFile, Guid.NewGuid().ToString()) { }

    /// <summary>
    /// Prepare a template for fast compilation.
    ///
    /// This will compile the document in development mode.
    /// Your template should not have any required inputs that don't have
    /// development or default values defined, otherwise registration will fail.
    /// 
    /// This call can be expensive depending on the template.
    /// Reuse instances of this class if possible.
    /// 
    /// If you want to compile a template once and not cache the template
    /// use <see cref="CompileOnce"/> instead.
    /// </summary>
    /// <param name="templateFile">The packed Oicana template to register.</param>
    /// <param name="templateId">Identifier of the template</param>
    /// <exception cref="OicanaException">If the initial template compilation fails.</exception>
    public Template(byte[] templateFile, string templateId) : this(templateFile, CompilationMode.Development, Guid.NewGuid().ToString()) { }

    /// <summary>
    /// Prepare a template for fast compilation.
    ///
    /// Your template should not require and explicit input values in
    /// the given compilation mode, otherwise registration will fail.
    /// 
    /// This call can be expensive depending on the template.
    /// Reuse instances of this class if possible.
    /// 
    /// If you want to compile a template once and not cache the template
    /// use <see cref="CompileOnce"/> instead.
    /// </summary>
    /// <param name="templateFile">The packed Oicana template to register.</param>
    /// <param name="compilationMode">Compilation mode to use for the initial template compilation during registration</param>
    /// <param name="templateId">Identifier of the template</param>
    /// <exception cref="OicanaException">If the initial template compilation fails.</exception>
    public Template(byte[] templateFile, CompilationMode compilationMode, string? templateId) : this(templateFile, new List<TemplateJsonInput>(), new List<TemplateBlobInput>(), CompilationMode.Development, Guid.NewGuid().ToString()) { }

    /// <summary>
    /// Prepare a template for fast compilation.
    /// 
    /// This call can be expensive depending on the template.
    /// Reuse instances of this class if possible.
    /// 
    /// If you want to compile a template once and not cache the template
    /// use <see cref="CompileOnce"/> instead.
    /// </summary>
    /// <param name="templateFile">The packed Oicana template to register.</param>
    /// <param name="jsonInputs">Json inputs for the initial compilation.</param>
    /// <param name="blobInputs">Blob inputs for the initial compilation.</param>
    /// <param name="compilationMode">Compilation mode to use for the initial template compilation during registration.</param>
    /// <param name="templateId">Identifier of the template.</param>
    /// <exception cref="OicanaException">If the initial template compilation fails.</exception>
    public Template(byte[] templateFile, IList<TemplateJsonInput> jsonInputs, IList<TemplateBlobInput> blobInputs, CompilationMode compilationMode, string? templateId)
    {
        _templateId = templateId ?? Guid.NewGuid().ToString();
        OicanaFfi.RegisterTemplate(_templateId, templateFile, jsonInputs, blobInputs, CompilationOptions.Pdf(compilationMode));
    }

    /// <inheritdoc />
    public Stream Compile(IList<TemplateJsonInput> jsonInputs, IList<TemplateBlobInput> blobInputs, CompilationOptions compilationOption)
    {
        return OicanaFfi.CompileTemplate(_templateId, jsonInputs, blobInputs, compilationOption);
    }

    /// <summary>
    /// Compile the given template once.
    /// </summary>
    /// <remarks>
    /// If you want to compile the same document multiple times with different input values,
    /// create an instance of <see cref="Template"/> and use <see cref="Compile(IList{TemplateJsonInput}, IList{TemplateBlobInput}, CompilationOptions)"/> instead.
    ///
    /// <see cref="CompileOnce"/> will use caching and thus be slower than compiling a prepared template.
    /// </remarks>
    /// <param name="templateFile">The packed Oicana template to compile.</param>
    /// <param name="jsonInputs">Json inputs for the compilation.</param>
    /// <param name="blobInputs">Blob inputs for the compilation.</param>
    /// <param name="compilationOptions">Options for the template compilation.</param>
    /// <exception cref="OicanaException">If the template compilation fails.</exception>
    public static Stream CompileOnce(byte[] templateFile, IList<TemplateJsonInput> jsonInputs, IList<TemplateBlobInput> blobInputs, CompilationOptions compilationOptions)
    {
        return OicanaFfi.CompileTemplateOnce(templateFile, jsonInputs, blobInputs, compilationOptions);
    }

    /// <inheritdoc/>
    public void Dispose()
    {
        OicanaFfi.ResetTemplate(_templateId);
    }

    /// <inheritdoc/>
    public override string ToString() =>
        $"Template {_templateId}";
}
