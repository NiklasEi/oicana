using System.IO.Compression;
using System.Text;
using System.Text.Json.Nodes;
using AwesomeAssertions;
using Oicana.Inputs;
using CompilationMode = Oicana.Config.CompilationMode;
using CompilationOptions = Oicana.Config.CompilationOptions;
using ExportFormat = Oicana.Config.ExportFormat;

namespace Oicana.Test;

public class TemplateWarningsTests
{
    private const string MinimalManifest = """
        [package]
        name = "template-warnings-test"
        version = "0.1.0"
        entrypoint = "main.typ"

        [tool.oicana]
        manifest_version = 1
        """;

    private static byte[] PackTemplate(string manifest, string mainTypst)
    {
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

    [Fact]
    public void ExportSurfacesWarnings()
    {
        var templateFile = PackTemplate(
            MinimalManifest,
            "#set text(font: \"NonexistentFontTemplate\")\nContent");
        using var template = new Template(templateFile);

        // Constructor warm-up compile already warns.
        template.Warnings.Should().NotBeNull();

        using var document = template.ExportSvg(
            new Dictionary<string, JsonNode>(),
            new Dictionary<string, BlobInput>(),
            new CompilationOptions(CompilationMode.Development));

        template.Warnings.Should().NotBeNull();
        template.Warnings.Should().Contain("NonexistentFontTemplate");
    }

    [Fact]
    public void ExportWithoutWarningsLeavesNull()
    {
        var templateFile = PackTemplate(MinimalManifest, "Content");
        using var template = new Template(templateFile);

        template.Warnings.Should().BeNull();

        using var document = template.ExportSvg(
            new Dictionary<string, JsonNode>(),
            new Dictionary<string, BlobInput>(),
            new CompilationOptions(CompilationMode.Development));

        template.Warnings.Should().BeNull();
    }
}
