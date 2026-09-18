using System.IO.Compression;
using System.Text;
using System.Text.Json.Nodes;
using AwesomeAssertions;
using Microsoft.Extensions.Logging.Abstractions;
using Oicana.Inputs;
using CompilationMode = Oicana.Config.CompilationMode;
using CompilationOptions = Oicana.Config.CompilationOptions;

namespace Oicana.Test;

public class OicanaServiceTests
{
    private static byte[] PackTemplate(string pageWidth)
    {
        const string manifest = """
            [package]
            name = "service-test"
            version = "0.1.0"
            entrypoint = "main.typ"

            [tool.oicana]
            manifest_version = 1
            """;
        var mainTypst = $"#set page(width: {pageWidth}, height: 100pt)\nContent";

        using var stream = new MemoryStream();
        using (var zip = new ZipArchive(stream, ZipArchiveMode.Create, leaveOpen: true))
        {
            foreach (var (name, content) in new[] { ("typst.toml", manifest), ("main.typ", mainTypst) })
            {
                var entry = zip.CreateEntry(name, CompressionLevel.NoCompression);
                using var entryStream = entry.Open();
                var bytes = Encoding.UTF8.GetBytes(content);
                entryStream.Write(bytes, 0, bytes.Length);
            }
        }

        return stream.ToArray();
    }

    private static OicanaService CreateService() => new(NullLogger<OicanaService>.Instance);

    private static string ExportSvg(ITemplate template)
    {
        using var svg = template.Export(
            new Dictionary<string, JsonNode>(),
            new Dictionary<string, BlobInput>(),
            Config.ExportFormat.Svg(),
            new CompilationOptions(CompilationMode.Development));
        using var reader = new StreamReader(svg);
        return reader.ReadToEnd();
    }

    [Fact]
    public void RegisteringAKnownIdReplacesAndDisposesThePreviousTemplate()
    {
        using var service = CreateService();
        service.RegisterTemplate("invoice", PackTemplate("100pt"));
        var replaced = service.GetTemplate("invoice");

        service.RegisterTemplate("invoice", PackTemplate("200pt"));

        var template = service.GetTemplate("invoice");
        template.Should().NotBeNull();
        ExportSvg(template!).Should().Contain("200pt");
        var export = () => ExportSvg(replaced!);
        export.Should().Throw<OicanaException>();
    }

    [Fact]
    public void DisposingTheServiceDisposesEveryRegisteredTemplate()
    {
        var service = CreateService();
        service.RegisterTemplate("first", PackTemplate("100pt"));
        service.RegisterTemplate("second", PackTemplate("200pt"));
        var first = service.GetTemplate("first");
        var second = service.GetTemplate("second");

        service.Dispose();

        service.GetTemplate("first").Should().BeNull();
        var exportFirst = () => ExportSvg(first!);
        exportFirst.Should().Throw<OicanaException>();
        var exportSecond = () => ExportSvg(second!);
        exportSecond.Should().Throw<OicanaException>();
    }

    [Fact]
    public void RemovedTemplatesCanBeDisposedThroughTheInterface()
    {
        using var service = CreateService();
        service.RegisterTemplate("invoice", PackTemplate("100pt"));

        ITemplate? removed = service.RemoveTemplate("invoice");

        removed.Should().NotBeNull();
        service.GetTemplate("invoice").Should().BeNull();
        removed!.Dispose();
    }
}
