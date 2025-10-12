namespace Oicana.Config;

/// <summary>
/// Options for compiling an Oicana template
/// </summary>
public class CompilationOptions
{
    internal readonly CompilationMode CompilationMode;

    /// <summary>
    /// Create new compilation options
    /// </summary>
    /// <param name="compilationMode">Mode to compile the template in.</param>
    public CompilationOptions(CompilationMode compilationMode)
    {
        CompilationMode = compilationMode;
    }
}
