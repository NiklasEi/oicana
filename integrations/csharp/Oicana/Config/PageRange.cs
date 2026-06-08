namespace Oicana.Config;

/// <summary>
/// A contiguous, 1-based inclusive range of document pages to export.
///
/// Both bounds are optional; leaving one as <c>null</c> keeps it open. For
/// example, <see cref="Of"/> with <c>start: 2</c> selects page 2 to the end of
/// the document.
/// </summary>
public class PageRange
{
    internal int? Start;
    internal int? End;

    /// <summary>
    /// Create a range selecting exactly the given 1-based page.
    /// </summary>
    /// <param name="page">The 1-based page to select.</param>
    public static PageRange Single(int page)
    {
        return new PageRange()
        {
            Start = page,
            End = page
        };
    }

    /// <summary>
    /// Create a range with the given (nullable) 1-based, inclusive bounds.
    /// </summary>
    /// <param name="start">The first page to export, or <c>null</c> to start at the first page.</param>
    /// <param name="end">The last page to export, or <c>null</c> to go to the last page.</param>
    public static PageRange Of(int? start = null, int? end = null)
    {
        return new PageRange()
        {
            Start = start,
            End = end
        };
    }
}
