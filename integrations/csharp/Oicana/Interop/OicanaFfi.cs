using System.Runtime.InteropServices;
using System.Text;
using System.Text.RegularExpressions;
using Oicana.Config;
using Oicana.Inputs;

namespace Oicana.Interop;

/// <summary>
/// Compile Oicana templates
/// </summary>
internal static class OicanaFfi
{
    /// <summary>
    /// Compile the given template once and do not cache anything
    /// 
    /// This method does a clean compile which can take significantly longer
    /// than compiling a template through the `Template` class.
    /// If you want to compile a template multiple times with different
    /// inputs use the Template class!
    /// </summary>
    /// <param name="templateFile">The packed Oicana template to compile.</param>
    /// <param name="jsonInputs">Json inputs for the compilation.</param>
    /// <param name="blobInputs">Blob inputs for the compilation.</param>
    /// <param name="compilationOptions">Options for the template compilation.</param>
    /// <param name="exportOptions">Options for the document export.</param>
    /// <exception cref="OicanaException">If the template compilation fails.</exception>
    /// <returns>Stream containing the compiled template exported as the given <see cref="ExportTarget"/>.</returns>
    public static Stream ExportTemplateOnce(byte[] templateFile, IList<TemplateJsonInput> jsonInputs, IList<TemplateBlobInput> blobInputs, Oicana.Config.CompilationOptions compilationOptions, Oicana.Config.ExportOptions exportOptions)
    {
        GCHandle fileHandle = GCHandle.Alloc(templateFile, GCHandleType.Pinned);
        IntPtr filePointer = fileHandle.AddrOfPinnedObject();
        var fileBuffer = new Buffer() { data = filePointer, error = false, len = (uint)templateFile.Length };

        PreparedInputs preparedInputs = PrepareInputs(jsonInputs, blobInputs);

        var buffer = OicanaFfiInternal.unsafe_export_template_once(fileBuffer, preparedInputs.JsonInputs, preparedInputs.BlobInputs, ConvertCompileOptions(compilationOptions), ConvertExportOptions(exportOptions));

        preparedInputs.FreeAll();
        fileHandle.Free();

        return HandleBuffer(buffer);
    }

    /// <summary>
    /// Compile a template with the given id and inputs and export it to the specified <see cref="ExportTarget"/>.
    /// </summary>
    /// <param name="templateId">Identifier of the template for the internal cache.</param>
    /// <param name="jsonInputs">Json inputs for the compilation.</param>
    /// <param name="blobInputs">Blob inputs for the compilation.</param>
    /// <param name="compilationOptions">Options for the template compilation.</param>
    /// <exception cref="OicanaException">If the template compilation fails.</exception>
    /// <returns>Stream containing the compiled template exported as the given <see cref="ExportTarget"/>.</returns>
    public static String CompileTemplate(string templateId, IList<TemplateJsonInput> jsonInputs, IList<TemplateBlobInput> blobInputs, Oicana.Config.CompilationOptions compilationOptions)
    {
        PreparedInputs preparedInputs = PrepareInputs(jsonInputs, blobInputs);

        var buffer = OicanaFfiInternal.unsafe_compile_template(templateId, preparedInputs.JsonInputs, preparedInputs.BlobInputs, ConvertCompileOptions(compilationOptions));

        preparedInputs.FreeAll();

        return HandleStringBuffer(buffer);
    }

    /// <summary>
    /// Register and compile a template with the given id and inputs and export it to the specified <see cref="ExportTarget"/>.
    /// </summary>
    /// <param name="templateId">Identifier of the template for the internal cache.</param>
    /// <param name="templateFile">The packed Oicana template to compile.</param>
    /// <param name="jsonInputs">Json inputs for the compilation.</param>
    /// <param name="blobInputs">Blob inputs for the compilation.</param>
    /// <param name="compilationOptions">Options for the template compilation.</param>
    /// <exception cref="OicanaException">If the template compilation fails.</exception>
    /// <returns>Stream containing the compiled template exported as the given <see cref="ExportTarget"/>.</returns>
    public static Stream RegisterTemplate(string templateId, byte[] templateFile, IList<TemplateJsonInput> jsonInputs, IList<TemplateBlobInput> blobInputs, Oicana.Config.CompilationOptions compilationOptions)
    {
        GCHandle fileHandle = GCHandle.Alloc(templateFile, GCHandleType.Pinned);
        IntPtr filePointer = fileHandle.AddrOfPinnedObject();
        var fileBuffer = new Buffer() { data = filePointer, error = false, len = (uint)templateFile.Length };

        PreparedInputs preparedInputs = PrepareInputs(jsonInputs, blobInputs);

        var buffer = OicanaFfiInternal.unsafe_register_template(templateId, fileBuffer, preparedInputs.JsonInputs, preparedInputs.BlobInputs, ConvertCompileOptions(compilationOptions));

        preparedInputs.FreeAll();
        fileHandle.Free();

        return HandleBuffer(buffer);
    }

    /// <summary>
    /// Export the given document
    /// 
    /// This method requires the document to be in the internal cache.
    /// After the export it will not be removed from the cache automatically. It's the callers
    /// responsibility to free the documents memory when no more exports are needed by calling
    /// `RemoveDocument`.
    /// </summary>
    /// <param name="documentId">Id of document to export.</param>
    /// <param name="exportOptions">Options for the document export.</param>
    /// <exception cref="OicanaException">If the template compilation fails.</exception>
    /// <returns>Stream containing the compiled template exported as the given <see cref="ExportTarget"/>.</returns>
    public static Stream ExportDocument(string documentId, Oicana.Config.ExportOptions exportOptions)
    {
        var buffer = OicanaFfiInternal.unsafe_export_document(documentId, ConvertExportOptions(exportOptions));

        return HandleBuffer(buffer);
    }

    /// <summary>
    /// Reset the world cache of the given template id
    /// </summary>
    /// <param name="id">The identifier of the template to reset.</param>
    public static void ResetTemplate(string id)
    {
        OicanaFfiInternal.remove_world(id);
    }

    /// <summary>
    /// Remove a document from the internal cache
    /// </summary>
    /// <param name="documentId">The identifier of the document to remove from the cache.</param>
    public static void RemoveDocument(string documentId)
    {
        OicanaFfiInternal.remove_document(documentId);
    }

    /// <summary>
    /// Set the global cache age for comemo cache eviction.
    ///
    /// # How Cache Aging Works
    ///
    /// - Each cache entry has an age counter
    /// - Age increases by 1 during each eviction call
    /// - Age resets to 0 when the entry is used (cache hit)
    /// - Entries with age >= `maxAge` are removed
    ///
    /// Default: 10
    /// </summary>
    /// <param name="maxAge">
    /// Maximum age threshold, or -1 to disable:
    ///   - `-1` - Disables cache eviction (cache never cleared)
    ///   - `0` - Clears all cache after every compilation
    ///   - `1` - Keeps only entries used since the last eviction
    ///   - `n` - Keeps entries used within the last n evictions
    /// </param>
    public static void ConfigureAutomaticCacheEviction(long maxAge)
    {
        OicanaFfiInternal.configure_automatic_cache_eviction(maxAge);
    }

    /// <summary>
    /// Manually evict the comemo cache based on the configured cache age.
    /// </summary>
    /// <param name="maxAge">
    /// Maximum age threshold for cache eviction.
    /// Entries with age >= this value will be removed.
    /// Pass -1 to disable eviction.
    /// </param>
    public static void EvictCache(long maxAge)
    {
        OicanaFfiInternal.evict_cache(maxAge);
    }

    /// <summary>
    /// Configure Oicana.
    /// </summary>
    /// <param name="coloring">Coloring for Oicana diagnostics.</param>
    public static void Configure(DiagnosticsColoring coloring)
    {
        DiagnosticColor color;
        switch (coloring)
        {
            case DiagnosticsColoring.Ansi:
                {
                    color = DiagnosticColor.Ansi;
                }
                break;
            default:
                {
                    color = DiagnosticColor.None;
                }
                break;
        }

        OicanaFfiInternal.configure(new Config
        {
            color = color,
        });
    }

    private record PreparedInputs(IntPtr JsonInputsPtr, SliceFfiJsonInput JsonInputs, IntPtr BlobsInputsPtr, SliceFfiBlobInput BlobInputs, List<GCHandle> BlobHandles)
    {
        internal readonly IntPtr JsonInputsPtr = JsonInputsPtr;
        internal readonly SliceFfiJsonInput JsonInputs = JsonInputs;

        internal readonly IntPtr BlobsInputsPtr = BlobsInputsPtr;
        internal readonly SliceFfiBlobInput BlobInputs = BlobInputs;
        internal readonly List<GCHandle> BlobHandles = BlobHandles;

        internal void FreeAll()
        {
            Marshal.FreeHGlobal(JsonInputsPtr);
            Marshal.FreeHGlobal(BlobsInputsPtr);
            foreach (var handle in BlobHandles)
            {
                handle.Free();
            }
        }
    }

    internal static Oicana.Interop.CompilationMode ConvertCompilationMode(Oicana.Config.CompilationMode compilationMode)
    {
        switch (compilationMode)
        {
            case Oicana.Config.CompilationMode.Development:
                return Oicana.Interop.CompilationMode.Development;
            case Oicana.Config.CompilationMode.Production:
                return Oicana.Interop.CompilationMode.Production;
        }
        throw new ArgumentException($"The compilation mode {nameof(compilationMode)} is not supported.");
    }

    internal static Oicana.Interop.CompilationOptions ConvertCompileOptions(
        Oicana.Config.CompilationOptions compilationOptions)
    {
        return new CompilationOptions()
        {
            mode = ConvertCompilationMode(compilationOptions.CompilationMode),
        };
    }

    internal static Oicana.Interop.ExportOptions ConvertExportOptions(
        Oicana.Config.ExportOptions exportOptions)
    {
        return new ExportOptions()
        {
            target = ConvertCompileTarget(exportOptions.ExportTarget),
            px_per_pt = exportOptions.PixelsPerPt ?? 1.0f
        };
    }

    internal static Oicana.Interop.CompilationTarget ConvertCompileTarget(Oicana.Config.ExportTarget exportTarget)
    {
        switch (exportTarget)
        {
            case Oicana.Config.ExportTarget.Pdf:
                return Oicana.Interop.CompilationTarget.Pdf;
            case Oicana.Config.ExportTarget.Png:
                return Oicana.Interop.CompilationTarget.Png;
            case Oicana.Config.ExportTarget.Svg:
                return Oicana.Interop.CompilationTarget.Svg;
        }
        throw new ArgumentException($"The compile target {nameof(exportTarget)} is not supported.");
    }

    private static PreparedInputs PrepareInputs(IList<TemplateJsonInput> jsonInputs,
        IList<TemplateBlobInput> blobInputs)
    {
        IntPtr blobsInputsPtr = PrepareBlobInputs(blobInputs, out var blobHandles);
        var blobs = new SliceFfiBlobInput(blobsInputsPtr, (ulong)blobInputs.Count());

        IntPtr inputsPtr = PrepareJsonInputs(jsonInputs);
        var inputs = new SliceFfiJsonInput(inputsPtr, (ulong)jsonInputs.Count());

        return new PreparedInputs(inputsPtr, inputs, blobsInputsPtr, blobs, blobHandles);
    }

    private static IntPtr PrepareBlobInputs(IList<TemplateBlobInput> blobs, out List<GCHandle> blobHandles)
    {
        blobHandles = new List<GCHandle>();
        var blobsInputsPtr = Marshal.AllocHGlobal(blobs.Count * Marshal.SizeOf(typeof(FfiBlobInput)));
        for (int i = 0; i < blobs.Count; i++)
        {
            var blob = blobs.ElementAt(i);
            GCHandle blobHandle = GCHandle.Alloc(blob.Blob, GCHandleType.Pinned);
            IntPtr dataPtr = blobHandle.AddrOfPinnedObject();
            blobHandles.Add(blobHandle);

            var blobInput = new FfiBlobInput() { key = blob.Key, data = new Buffer() { data = dataPtr, error = false, len = (uint)blob.Blob.Length }, meta = blob.Meta?.ToString() ?? "{}" };
            Marshal.StructureToPtr(blobInput, blobsInputsPtr + i * Marshal.SizeOf(typeof(FfiBlobInput)), false);
        }

        return blobsInputsPtr;
    }

    private static IntPtr PrepareJsonInputs(IList<TemplateJsonInput> inputs)
    {
        var inputsPtr = Marshal.AllocHGlobal(inputs.Count * Marshal.SizeOf(typeof(FfiJsonInput)));
        for (int i = 0; i < inputs.Count; i++)
        {
            FfiJsonInput jsonInput = new FfiJsonInput { data = inputs[i].Value.ToString(), key = inputs[i].Key };
            Marshal.StructureToPtr(jsonInput, inputsPtr + i * Marshal.SizeOf(typeof(FfiJsonInput)), false);
        }

        return inputsPtr;
    }

    private static String HandleStringBuffer(Buffer buffer)
    {
        var message = GetStringFromBuffer(buffer);

        if (buffer.error)
        {
            throw new OicanaException(message);
        }

        return message;
    }

    private static Stream HandleBuffer(Buffer buffer)
    {
        if (buffer.error)
        {
            unsafe
            {
                UnmanagedMemoryStream errorStream = new UnmanagedMemoryStream((byte*)buffer.data.ToPointer(), buffer.len,
                    buffer.len, FileAccess.Read);
                var error = GetMessageFromStream(errorStream);
                OicanaFfiInternal.unsafe_free_buffer(buffer);
                throw new OicanaException(error);
            }
        }

        return new RustMemoryStream(buffer);
    }

    public static string GetStringFromBuffer(Buffer buffer)
    {
        unsafe
        {
            try
            {
                UnmanagedMemoryStream stream = new UnmanagedMemoryStream((byte*)buffer.data.ToPointer(),
                    buffer.len,
                    buffer.len, FileAccess.Read);
                var message = GetMessageFromStream(stream);
                OicanaFfiInternal.unsafe_free_buffer(buffer);
                return message;
            }
            catch (Exception ex)
            {
                return $"Failed to get string from Rust: {ex.Message}";
            }
        }
    }

    public static string GetMessageFromStream(Stream stream)
    {
        try
        {
            stream.Seek(0, SeekOrigin.Begin);
            byte[] buffer = new byte[stream.Length];
            stream.ReadExactly(buffer, 0, (int)stream.Length);
            var rawString = Encoding.UTF8.GetString(buffer);
            return Regex.Unescape(rawString);
        }
        catch (Exception ex)
        {
            return $"Unknown error during template compilation. Failed to read error message: {ex.Message}";
        }
    }
}
