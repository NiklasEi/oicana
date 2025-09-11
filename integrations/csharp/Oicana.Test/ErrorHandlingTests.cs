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
    public void ReadSimpleEscapes()
    {
        var stream = new MemoryStream("{ \\\"test\\\"\\n"u8.ToArray());
        var error = OicanaFfi.GetMessageFromStream(stream);
        error.Should().Be("{ \"test\"\n");
    }
}
