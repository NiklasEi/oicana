namespace Oicana.Config;

/// <summary>
/// Formats that an Oicana template can be compiled into.
/// </summary>
public enum CompilationTarget
{
    /// <summary>
    /// Render the template to a PDF file.
    ///
    /// The exported standard is PDF/A-3b
    /// </summary>
    Pdf = 0,
    /// <summary>
    /// Render the template into a png image.
    /// </summary>
    /// <remarks>The image is not optimized for file size to speed up compilation.</remarks>
    Png = 1,
    /// <summary>
    /// Render the template as SVG file.
    /// </summary>
    Svg = 2,
}
