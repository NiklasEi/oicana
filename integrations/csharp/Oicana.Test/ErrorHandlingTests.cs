using AwesomeAssertions;
using Oicana.Interop;

namespace Oicana.Test;

public class ErrorHandlingTests
{
    [Fact]
    public void ReadPlainError()
    {
        var stream = new MemoryStream("Hello World"u8.ToArray());
        var error = OicanaFfi.GetMessageFromStream(stream);
        error.Should().Be("Hello World");
    }

    [Fact]
    public void ReadBackslashSequencesVerbatim()
    {
        var stream = new MemoryStream("{ \\\"test\\\"\\n"u8.ToArray());
        var error = OicanaFfi.GetMessageFromStream(stream);
        error.Should().Be("{ \\\"test\\\"\\n");
    }

    [Fact]
    public void RefusesTemplatePackedByNewerOicana()
    {
        var templateFile = File.ReadAllBytes(
            Path.GetFullPath("../../../../../../assets/templates/future-manifest-0.1.0.zip"));

        var register = () => new Template(templateFile);

        register.Should().Throw<Exception>().WithMessage("*manifest_version 99*");
    }
}
