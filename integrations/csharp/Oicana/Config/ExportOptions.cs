namespace Oicana.Config;

/// <summary>
/// Options for exporting an Oicana template
/// </summary>
public class ExportOptions
{
    internal ExportTarget ExportTarget;
    internal float? PixelsPerPt;

    /// <summary>
    /// Create new compilation options for compiling to PDF
    /// </summary>
    public static ExportOptions Pdf()
    {
        return new ExportOptions()
        {
            ExportTarget = ExportTarget.Pdf,
        };
    }

    /// <summary>
    /// Create new compilation options for compiling to PNG
    /// </summary>
    /// <param name="pixelsPerPt">The number of pixels per pt. Higher numbers take longer, but create sharper images.</param>
    public static ExportOptions Png(float pixelsPerPt = 1.0f)
    {
        return new ExportOptions()
        {
            ExportTarget = ExportTarget.Png,
            PixelsPerPt = pixelsPerPt
        };
    }

    /// <summary>
    /// Create new compilation options for compiling to SVG
    /// </summary>
    public static ExportOptions Svg()
    {
        return new ExportOptions()
        {
            ExportTarget = ExportTarget.Svg,
        };
    }
}
