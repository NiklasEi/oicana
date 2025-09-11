namespace Oicana.Template;

/// <summary>
/// A service for using Oicana templates
/// </summary>
public interface IOicanaService
{
    /// <summary>
    /// Register a <see cref="Template"/> for the given id based on the given template file.
    /// </summary>
    /// <param name="id">Identifier for this registration.</param>
    /// <param name="file">The packed Oicana template file.</param>
    void RegisterTemplate(string id, byte[] file);

    /// <summary>
    /// Get the template registered under the given id.
    /// </summary>
    /// <param name="id">Identifier of the template.</param>
    /// <returns><see cref="Template"/> if id is registered, <see langword="null"/> if the id is not registered.</returns>
    ITemplate? GetTemplate(string id);

    /// <summary>
    /// Remove the template registered under the given id from the service.
    /// </summary>
    /// <param name="id">Identifier of the template.</param>
    /// <returns><see cref="Template"/> if id is registered, <see langword="null"/> if the id is not registered.</returns>
    ITemplate? RemoveTemplate(string id);
}
