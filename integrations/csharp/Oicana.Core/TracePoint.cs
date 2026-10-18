namespace Oicana;

/// <summary>
/// A step in the function calls leading to a <see cref="Diagnostic"/>.
/// </summary>
/// <param name="Message">What happened at this step, like <c>error occurred in this function call</c>.</param>
/// <param name="Location">Where the step happened, or <c>null</c> if it does not point into a file.</param>
public sealed record TracePoint(string Message, SourceLocation? Location);
