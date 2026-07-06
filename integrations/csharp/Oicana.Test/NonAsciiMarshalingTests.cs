using System.IO.Compression;
using System.Text;
using System.Text.Json.Nodes;
using AwesomeAssertions;
using Oicana.Inputs;
using Oicana.Interop;
using CompilationMode = Oicana.Config.CompilationMode;
using CompilationOptions = Oicana.Config.CompilationOptions;

namespace Oicana.Test;

/// <summary>
/// Regression tests for non-ASCII strings crossing the FFI boundary.
/// </summary>
public class NonAsciiMarshalingTests
{
    private const string NonAsciiText = "Müller Grüße 你好 🚀 Ω €";

    private static string ManifestWithJsonInput(string key) => $"""
        [package]
        name = "non-ascii-test"
        version = "0.1.0"
        entrypoint = "main.typ"

        [tool.oicana]
        manifest_version = 1

        [[tool.oicana.inputs]]
        type = "json"
        key = '{key}'
        required = false
        """;

    private static string EchoingMain(string key) => $$"""
        #let ins = sys.inputs.at("oicana-inputs", default: (:))
        #if "{{key}}" in ins {
          assert(false, message: "ECHO<" + json(bytes(ins.at("{{key}}"))).text + ">")
        }
        Warm-up content
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

    private static void CompileExpectingEcho(Template template, string inputKey)
    {
        var act = () => template.Compile(
            new Dictionary<string, JsonNode> { [inputKey] = new JsonObject { ["text"] = NonAsciiText } },
            new Dictionary<string, BlobInput>(),
            new CompilationOptions(CompilationMode.Development));

        act.Should().Throw<OicanaException>()
            .Which.Message.Should().Contain($"ECHO<{NonAsciiText}>",
                "the template must receive the input value byte-for-byte as UTF-8");
    }

    [Fact]
    public void JsonInputValueWithNonAsciiTextReachesTheTemplateIntact()
    {
        using var template = new Template(PackTemplate(ManifestWithJsonInput("echo"), EchoingMain("echo")));

        CompileExpectingEcho(template, "echo");
    }

    [Fact]
    public void JsonInputKeyWithNonAsciiCharactersIsMatchedByTheTemplate()
    {
        const string key = "grüße";
        using var template = new Template(PackTemplate(ManifestWithJsonInput(key), EchoingMain(key)));

        CompileExpectingEcho(template, key);
    }

    [Fact]
    public void TemplateIdWithNonAsciiCharactersWorksAcrossAllCalls()
    {
        var packed = PackTemplate(ManifestWithJsonInput("unused"), "Hello");
        using var template = new Template(packed, $"grüße-🚀-{Guid.NewGuid()}");

        template.Source("main.typ").Should().Be("Hello");
        JsonNode.Parse(template.Inputs()).Should().NotBeNull();

        using var document = template.Compile(
            new Dictionary<string, JsonNode>(),
            new Dictionary<string, BlobInput>(),
            new CompilationOptions(CompilationMode.Development));
        document.PageCount.Should().BeGreaterThan(0);
    }
}
