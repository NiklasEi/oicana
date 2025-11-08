using System.Text.Json;
using System.Text.Json.Nodes;
using AwesomeAssertions;
using Oicana.Inputs;
using Oicana.Interop;
using CompilationMode = Oicana.Config.CompilationMode;
using CompilationOptions = Oicana.Config.CompilationOptions;
using ExportOptions = Oicana.Config.ExportOptions;

namespace Oicana.Test;

using Oicana.Template;

public class E2ETests
{
    private readonly byte[] _templateFile = File.ReadAllBytes("../../../../../../e2e-tests/template/oicana-e2e-test-x.y.z.zip");

    public E2ETests()
    {
        Directory.CreateDirectory("e2e");
    }

    [Fact]
    public void Development()
    {
        var template = new Template(_templateFile);

        var document = template.Compile(new List<TemplateJsonInput>(), new List<TemplateBlobInput>(), new CompilationOptions(CompilationMode.Development), ExportOptions.Png(1.0f));
        using var fileStream = File.Create("e2e/development.png");
        document.CopyTo(fileStream);
    }

    [Fact]
    public void Production()
    {
        var template = new Template(_templateFile);

        var blobInputs = new List<TemplateBlobInput>()
        {
            new ("development-blob", "Input"u8.ToArray(), new BlobMeta()
            {
                ImageFormat = "jpeg",
                Custom = JsonNode.Parse(
                    """
                    {
                       "bar": ["input", "two"],
                       "foo": 43
                    }
                    """)!.AsObject()
            }),
        };
        var input = new TemplateJsonInput("development-json", JsonSerializer.Deserialize<JsonNode>("{ \"name\": \"Input\", \"foo\": [41, \"testing\"] }")!);
        var document = template.Compile([input], blobInputs, new CompilationOptions(CompilationMode.Production), ExportOptions.Png(1.0f));
        using var fileStream = File.Create("e2e/production.png");
        document.CopyTo(fileStream);
    }

    [Fact]
    public void AllInputs()
    {
        var template = new Template(_templateFile);

        var blobInputs = new List<TemplateBlobInput>()
        {
            new ("default-blob", "Input"u8.ToArray(), new BlobMeta()
            {
                ImageFormat = "jpeg",
                Custom = JsonNode.Parse(
                    """
                    {
                       "bar": ["input", "two"],
                       "foo": 42
                    }
                    """)!.AsObject()
            }),
            new ("development-blob", "Input"u8.ToArray(), new BlobMeta()
            {
                ImageFormat = "jpeg",
                Custom = JsonNode.Parse(
                    """
                    {
                       "bar": ["input", "two"],
                       "foo": 43
                    }
                    """)!.AsObject()
            }),
            new ("both-blob", "Input"u8.ToArray(), new BlobMeta()
            {
                ImageFormat = "jpeg",
                Custom = JsonNode.Parse(
                    """
                    {
                       "bar": ["input", "two"],
                       "foo": 44
                    }
                    """)!.AsObject()
            }),
        };
        var jsonInputs = new List<TemplateJsonInput>()
        {
            new("default-json", JsonSerializer.Deserialize<JsonNode>("{ \"name\": \"Input\", \"foo\": [41, \"testing\"] }")!),
            new("development-json", JsonSerializer.Deserialize<JsonNode>("{ \"name\": \"Input\", \"foo\": [41, \"testing\"] }")!),
            new("both-json", JsonSerializer.Deserialize<JsonNode>("{ \"name\": \"Input\", \"foo\": [41, \"testing\"] }")!)
        };

        var document = template.Compile(jsonInputs, blobInputs, new CompilationOptions(CompilationMode.Production), ExportOptions.Png(1.0f));
        using var fileStream = File.Create("e2e/all-inputs.png");
        document.CopyTo(fileStream);
    }

    [Fact]
    public void GetsReadableErrors()
    {
        var template = new Template(_templateFile);
        Action act = () => template.Compile(new List<TemplateJsonInput>(), new List<TemplateBlobInput>(), new CompilationOptions(CompilationMode.Production), ExportOptions.Png(1.0f));

        act.Should()
            .Throw<OicanaException>()
            .WithMessage("error: dictionary does not contain key \"development-blob\"\n   \u250c\u2500 /main.typ:12:41\n   \u2502\n12 \u2502 `development-blob` has value: #str(input.development-blob.bytes)\n   \u2502                                          ^^^^^^^^^^^^^^^^\n\n");
    }
}
