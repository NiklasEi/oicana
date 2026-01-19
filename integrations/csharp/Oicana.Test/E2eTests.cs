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

        var document = template.Compile(new Dictionary<string, JsonNode>(), new Dictionary<string, BlobInput>(), ExportOptions.Png(1.0f), new CompilationOptions(CompilationMode.Development));
        using var fileStream = File.Create("e2e/development.png");
        document.CopyTo(fileStream);
    }

    [Fact]
    public void Production()
    {
        var template = new Template(_templateFile);

        var blobInputs = new Dictionary<string, BlobInput>
        {
            ["development-blob"] = new BlobInput("Input"u8.ToArray(), new BlobMeta()
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
        var jsonInputs = new Dictionary<string, JsonNode>
        {
            ["development-json"] = JsonSerializer.Deserialize<JsonNode>("{ \"name\": \"Input\", \"foo\": [41, \"testing\"] }")!
        };
        var document = template.Compile(jsonInputs, blobInputs, ExportOptions.Png(1.0f), new CompilationOptions(CompilationMode.Production));
        using var fileStream = File.Create("e2e/production.png");
        document.CopyTo(fileStream);
    }

    [Fact]
    public void AllInputs()
    {
        var template = new Template(_templateFile);

        var blobInputs = new Dictionary<string, BlobInput>
        {
            ["default-blob"] = new BlobInput("Input"u8.ToArray(), new BlobMeta()
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
            ["development-blob"] = new BlobInput("Input"u8.ToArray(), new BlobMeta()
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
            ["both-blob"] = new BlobInput("Input"u8.ToArray(), new BlobMeta()
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
        var jsonData = JsonSerializer.Deserialize<JsonNode>("{ \"name\": \"Input\", \"foo\": [41, \"testing\"] }")!;
        var jsonInputs = new Dictionary<string, JsonNode>
        {
            ["default-json"] = jsonData,
            ["development-json"] = jsonData,
            ["both-json"] = jsonData
        };

        var document = template.Compile(jsonInputs, blobInputs, ExportOptions.Png(1.0f), new CompilationOptions(CompilationMode.Production));
        using var fileStream = File.Create("e2e/all-inputs.png");
        document.CopyTo(fileStream);
    }

    [Fact]
    public void GetsReadableErrors()
    {
        var template = new Template(_templateFile);
        Action act = () => template.Compile(new Dictionary<string, JsonNode>(), new Dictionary<string, BlobInput>(), ExportOptions.Png(1.0f), new CompilationOptions(CompilationMode.Production));

        act.Should()
            .Throw<OicanaException>()
            .WithMessage("error: dictionary does not contain key \"development-blob\"\n   \u250c\u2500 /main.typ:12:41\n   \u2502\n12 \u2502 `development-blob` has value: #str(input.development-blob.bytes)\n   \u2502                                          ^^^^^^^^^^^^^^^^\n\n");
    }

    [Fact]
    public void GetInputs()
    {
        var template = new Template(_templateFile);

        var inputs = template.Inputs();

        inputs.Should().NotBeNullOrEmpty();
        var parsed = JsonSerializer.Deserialize<JsonNode>(inputs);
        parsed.Should().NotBeNull();

        inputs.Should().Contain("default-json");
        inputs.Should().Contain("development-json");
        inputs.Should().Contain("both-json");
        inputs.Should().Contain("default-blob");
        inputs.Should().Contain("development-blob");
        inputs.Should().Contain("both-blob");
    }

    [Fact]
    public void GetSource()
    {
        var template = new Template(_templateFile);

        var source = template.Source("/main.typ");

        source.Should().NotBeNullOrEmpty();
        source.Should().Contain("#import");
    }

    [Fact]
    public void GetSourceThrowsForMissingFile()
    {
        var template = new Template(_templateFile);
        Action act = () => template.Source("/nonexistent.typ");

        act.Should().Throw<OicanaException>();
    }

    [Fact]
    public void GetFile()
    {
        var template = new Template(_templateFile);

        var file = template.File("/default.txt");

        file.Should().NotBeNullOrEmpty();
        System.Text.Encoding.UTF8.GetString(file).Should().Contain("Default");
    }

    [Fact]
    public void GetFileThrowsForMissingFile()
    {
        var template = new Template(_templateFile);
        Action act = () => template.File("/nonexistent.png");

        act.Should().Throw<OicanaException>();
    }
}
