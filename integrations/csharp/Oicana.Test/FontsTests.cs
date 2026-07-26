using System.IO.Compression;
using System.Text;
using System.Text.Json.Nodes;
using AwesomeAssertions;
using Oicana.Inputs;
using CompilationMode = Oicana.Config.CompilationMode;
using CompilationOptions = Oicana.Config.CompilationOptions;
using ExportFormat = Oicana.Config.ExportFormat;

namespace Oicana.Test;

/// <summary>
/// Tests for host fonts registered through <see cref="Configuration"/>.
///
/// The font registry is process-global, so every test clears it around itself.
/// xUnit runs tests in one class sequentially, which keeps them from observing
/// each other's registrations.
/// </summary>
public class FontsTests : IDisposable
{
    public FontsTests() => Configuration.ClearFonts();

    public void Dispose() => Configuration.ClearFonts();

    private const string PlainManifest = """
        [package]
        name = "font-test"
        version = "0.1.0"
        entrypoint = "main.typ"

        [tool.oicana]
        manifest_version = 1
        """;

    private static string ManifestRequiring(string family) => $"""
        [package]
        name = "font-test"
        version = "0.1.0"
        entrypoint = "main.typ"

        [tool.oicana]
        manifest_version = 1

        [tool.oicana.fonts]
        require = ["{family}"]
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

    /// <summary>
    /// Family the test font provides. No system or Typst-embedded font has it, so a
    /// template requiring it can only be registered once the host registers the font.
    /// </summary>
    private const string TestFamily = "Oicana Test";

    /// <summary>The test font shipped with the repository.</summary>
    private static string AFontFile() =>
        Path.GetFullPath("../../../../../../assets/fonts/oicana-test-font.ttf");

    [Fact]
    public void RegistryStartsEmpty()
    {
        Configuration.RegisteredFonts().Should().BeEmpty();
    }

    [Fact]
    public void RegistersFontsFromBytesWithoutAPath()
    {
        var data = File.ReadAllBytes(AFontFile());

        Configuration.RegisterFonts(data).Should().Be(1);

        var fonts = Configuration.RegisteredFonts();
        fonts.Should().ContainSingle();
        fonts[0].Family.Should().Be(TestFamily);
        // Registered from memory, so no path is reported.
        fonts[0].Path.Should().BeNull();
    }

    [Fact]
    public void DataWithoutAFontIsIgnored()
    {
        Configuration.RegisterFonts(Encoding.UTF8.GetBytes("not a font")).Should().Be(0);
        Configuration.RegisteredFonts().Should().BeEmpty();
    }

    [Fact]
    public void RegistersFontsByPathAndReportsThePath()
    {
        var path = AFontFile();

        Configuration.RegisterFontPaths(path).Should().Be(1);

        var fonts = Configuration.RegisteredFonts();
        fonts.Should().ContainSingle();
        fonts[0].Family.Should().Be(TestFamily);
        fonts[0].Path.Should().Be(path);
    }

    [Fact]
    public void UnreadablePathsAreSkipped()
    {
        Configuration.RegisterFontPaths("/nonexistent/font.ttf").Should().Be(0);
        Configuration.RegisteredFonts().Should().BeEmpty();
    }

    [Fact]
    public void ClearFontsEmptiesTheRegistry()
    {
        Configuration.RegisterFontPaths(AFontFile());
        Configuration.RegisteredFonts().Should().NotBeEmpty();

        Configuration.ClearFonts();

        Configuration.RegisteredFonts().Should().BeEmpty();
    }

    [Fact]
    public void TemplateRequiringAnUnavailableFamilyIsRejected()
    {
        var templateFile = PackTemplate(ManifestRequiring("Nonexistent Host Family"), "Content");

        var register = () => new Template(templateFile);

        register.Should().Throw<OicanaException>().WithMessage("*Nonexistent Host Family*");
    }

    [Fact]
    public void TestTemplateIsRejectedUntilTheFontIsRegistered()
    {
        var templateFile = PackTemplate(ManifestRequiring(TestFamily), "Content");

        var register = () => new Template(templateFile);

        // Proves the family really is unavailable without the host font.
        register.Should().Throw<OicanaException>().WithMessage($"*{TestFamily}*");
    }

    [Fact]
    public void TemplateRequiringARegisteredFamilyCompiles()
    {
        Configuration.RegisterFontPaths(AFontFile());

        using var template = new Template(PackTemplate(ManifestRequiring(TestFamily), "Content"));

        using var svg = template.Export(
            new Dictionary<string, JsonNode>(),
            new Dictionary<string, BlobInput>(),
            ExportFormat.Svg(),
            new CompilationOptions(CompilationMode.Development));
        using var reader = new StreamReader(svg);
        reader.ReadToEnd().Should().Contain("<svg");
    }

    [Fact]
    public void RegisteredFontRendersWithoutAWarning()
    {
        Configuration.RegisterFontPaths(AFontFile());

        using var template = new Template(
            PackTemplate(PlainManifest, $"#set text(font: \"{TestFamily}\")\nContent"));

        using var svg = template.Export(
            new Dictionary<string, JsonNode>(),
            new Dictionary<string, BlobInput>(),
            ExportFormat.Svg(),
            new CompilationOptions(CompilationMode.Development));

        template.Warnings.Should().BeNull();
    }

    /// <summary>Without the host font, the same template falls back and warns.</summary>
    [Fact]
    public void UnregisteredFamilyWarns()
    {
        using var template = new Template(
            PackTemplate(PlainManifest, $"#set text(font: \"{TestFamily}\")\nContent"));

        using var svg = template.Export(
            new Dictionary<string, JsonNode>(),
            new Dictionary<string, BlobInput>(),
            ExportFormat.Svg(),
            new CompilationOptions(CompilationMode.Development));

        template.Warnings.Should().Contain(TestFamily);
    }
}
