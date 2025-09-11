namespace Oicana.Config;

/// <summary>
/// Options for compiling an Oicana template
/// </summary>
public class CompilationOptions
{
    internal CompilationMode compilationMode;
    internal CompilationTarget compilationTarget;
    internal float? pixelsPerPt;

    /// <summary>
    /// Create new compilation options for compiling to PDF
    /// </summary>
    /// <param name="mode">The compilation mode defines what fallback values can be used for template inputs.</param>
    public static CompilationOptions Pdf(CompilationMode mode = CompilationMode.Production)
    {
        return new CompilationOptions()
        {
            compilationTarget = CompilationTarget.Pdf,
            compilationMode = mode,
        };
    }

    /// <summary>
    /// Create new compilation options for compiling to PNG
    /// </summary>
    /// <param name="pixelsPerPt">The number of pixels per pt. Higher numbers take longer, but create sharper images.</param>
    /// <param name="mode">The compilation mode defines what fallback values can be used for template inputs.</param>
    public static CompilationOptions Png(float pixelsPerPt = 1.0f, CompilationMode mode = CompilationMode.Production)
    {
        return new CompilationOptions()
        {
            compilationTarget = CompilationTarget.Png,
            compilationMode = mode,
            pixelsPerPt = pixelsPerPt,
        };
    }

    /// <summary>
    /// Create new compilation options for compiling to SVG
    /// </summary>
    /// <param name="mode">The compilation mode defines what fallback values can be used for template inputs.</param>
    public static CompilationOptions Svg(CompilationMode mode = CompilationMode.Production)
    {
        return new CompilationOptions()
        {
            compilationTarget = CompilationTarget.Svg,
            compilationMode = mode,
        };
    }
}
