using System.IO.Compression;
using System.Text;
using System.Text.Json.Nodes;
using AwesomeAssertions;
using Oicana.Inputs;
using Oicana.Interop;
using CompilationMode = Oicana.Config.CompilationMode;
using CompilationOptions = Oicana.Config.CompilationOptions;
using ExportFormat = Oicana.Config.ExportFormat;
using ZipLimits = Oicana.Config.ZipLimits;

namespace Oicana.Test;

public class ExportOnceTests
{
    private readonly byte[] _templateFile = File.ReadAllBytes("../../../../../../e2e-tests/template/oicana-e2e-test-x.y.z.zip");

    private const string MinimalManifest = """
        [package]
        name = "export-once-test"
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
    public void ExportsWithoutWarnings()
    {
        var result = Template.ExportOnce(
            _templateFile,
            new Dictionary<string, JsonNode>(),
            new Dictionary<string, BlobInput>(),
            ExportFormat.Pdf(),
            new CompilationOptions(CompilationMode.Development));

        using var memory = new MemoryStream();
        result.Document.CopyTo(memory);
        var bytes = memory.ToArray();
        Encoding.ASCII.GetString(bytes, 0, 4).Should().Be("%PDF");
        result.Warnings.Should().BeNull();
    }

    [Fact]
    public void SurfacesWarnings()
    {
        var template = PackTemplate(
            MinimalManifest,
            "#set text(font: \"NonexistentFontExportOnce\")\nContent");

        var result = Template.ExportOnce(
            template,
            new Dictionary<string, JsonNode>(),
            new Dictionary<string, BlobInput>(),
            ExportFormat.Svg(),
            new CompilationOptions(CompilationMode.Development));

        result.Warnings.Should().NotBeNull();
        result.Warnings.Should().Contain("NonexistentFontExportOnce");
    }

    [Fact]
    public void EnforcesZipLimits()
    {
        var act = () => Template.ExportOnce(
            _templateFile,
            new Dictionary<string, JsonNode>(),
            new Dictionary<string, BlobInput>(),
            ExportFormat.Pdf(),
            new CompilationOptions(CompilationMode.Development),
            limits: new ZipLimits { MaxEntries = 1 });

        act.Should().Throw<OicanaException>().WithMessage("*entries*");
    }

    [Fact]
    public void RegistrationEnforcesZipLimits()
    {
        var act = () => new Template(
            _templateFile,
            new Dictionary<string, JsonNode>(),
            new Dictionary<string, BlobInput>(),
            CompilationMode.Development,
            limits: new ZipLimits { MaxEntries = 1 });

        act.Should().Throw<OicanaException>().WithMessage("*entries*");
    }

    [Fact]
    public void RejectsNegativeZipLimits()
    {
        var entries = () => new ZipLimits { MaxEntries = -1 };
        entries.Should().Throw<ArgumentOutOfRangeException>().WithMessage("*MaxEntries*");

        var bytes = () => new ZipLimits { MaxTotalDecompressedBytes = -8 };
        bytes.Should().Throw<ArgumentOutOfRangeException>()
            .WithMessage("*MaxTotalDecompressedBytes*");
    }

    [Fact]
    public void AcceptsZeroAndNullZipLimits()
    {
        new ZipLimits { MaxEntries = 0 }.MaxEntries.Should().Be(0);
        new ZipLimits().MaxEntries.Should().BeNull();
    }
}
